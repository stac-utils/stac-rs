use crate::Error;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use serde::de::DeserializeOwned;
use stac::Format;
use stac_api::{BlockingClient, Fields, Item, ItemCollection, Items, Search};
use stac_duckdb::Client;
use std::str::FromStr;
use tokio::runtime::Builder;

#[pyfunction]
#[pyo3(signature = (href, *, intersects=None, ids=None, collections=None, max_items=None, limit=None, bbox=None, datetime=None, include=None, exclude=None, sortby=None, filter=None, query=None, use_duckdb=None))]
#[allow(clippy::too_many_arguments)]
pub fn search(
    py: Python<'_>,
    href: String,
    intersects: Option<StringOrDict>,
    ids: Option<StringOrList>,
    collections: Option<StringOrList>,
    max_items: Option<usize>,
    limit: Option<u64>,
    bbox: Option<Vec<f64>>,
    datetime: Option<String>,
    include: Option<StringOrList>,
    exclude: Option<StringOrList>,
    sortby: Option<StringOrList>,
    filter: Option<StringOrDict>,
    query: Option<Py<PyDict>>,
    use_duckdb: Option<bool>,
) -> PyResult<Bound<'_, PyList>> {
    let items = search_items(
        href,
        intersects,
        ids,
        collections,
        max_items,
        limit,
        bbox,
        datetime,
        include,
        exclude,
        sortby,
        filter,
        query,
        use_duckdb,
    )?;
    pythonize::pythonize(py, &items)
        .map_err(PyErr::from)
        .and_then(|v| v.extract())
}

#[pyfunction]
#[pyo3(signature = (outfile, href, *, intersects=None, ids=None, collections=None, max_items=None, limit=None, bbox=None, datetime=None, include=None, exclude=None, sortby=None, filter=None, query=None, format=None, options=None, use_duckdb=None))]
#[allow(clippy::too_many_arguments)]
pub fn search_to(
    outfile: String,
    href: String,
    intersects: Option<StringOrDict>,
    ids: Option<StringOrList>,
    collections: Option<StringOrList>,
    max_items: Option<usize>,
    limit: Option<u64>,
    bbox: Option<Vec<f64>>,
    datetime: Option<String>,
    include: Option<StringOrList>,
    exclude: Option<StringOrList>,
    sortby: Option<StringOrList>,
    filter: Option<StringOrDict>,
    query: Option<Py<PyDict>>,
    format: Option<String>,
    options: Option<Vec<(String, String)>>,
    use_duckdb: Option<bool>,
) -> PyResult<usize> {
    let items = search_items(
        href,
        intersects,
        ids,
        collections,
        max_items,
        limit,
        bbox,
        datetime,
        include,
        exclude,
        sortby,
        filter,
        query,
        use_duckdb,
    )?;
    let format = format
        .map(|s| s.parse())
        .transpose()
        .map_err(Error::from)?
        .or_else(|| Format::infer_from_href(&outfile))
        .unwrap_or_default();
    let item_collection = ItemCollection::from(items);
    let count = item_collection.items.len();
    Builder::new_current_thread()
        .build()?
        .block_on(format.put_opts(
            outfile,
            serde_json::to_value(item_collection).map_err(Error::from)?,
            options.unwrap_or_default(),
        ))
        .map_err(Error::from)?;
    Ok(count)
}

#[allow(clippy::too_many_arguments)]
fn search_items(
    href: String,
    intersects: Option<StringOrDict>,
    ids: Option<StringOrList>,
    collections: Option<StringOrList>,
    max_items: Option<usize>,
    limit: Option<u64>,
    bbox: Option<Vec<f64>>,
    datetime: Option<String>,
    include: Option<StringOrList>,
    exclude: Option<StringOrList>,
    sortby: Option<StringOrList>,
    filter: Option<StringOrDict>,
    query: Option<Py<PyDict>>,
    use_duckdb: Option<bool>,
) -> PyResult<Vec<Item>> {
    let mut fields = Fields::default();
    if let Some(include) = include {
        fields.include = include.into();
    }
    if let Some(exclude) = exclude {
        fields.exclude = exclude.into();
    }
    let fields = if fields.include.is_empty() && fields.exclude.is_empty() {
        None
    } else {
        Some(fields)
    };
    let query = Python::with_gil(|py| {
        query
            .map(|q| pythonize::depythonize(&q.into_bound(py)))
            .transpose()
    })?;
    let mut search = Search {
        intersects: intersects.map(|i| i.into()).transpose()?,
        ids: ids.map(|ids| ids.into()),
        collections: collections.map(|c| c.into()),
        items: Items {
            limit,
            bbox: bbox
                .map(|b| b.try_into())
                .transpose()
                .map_err(Error::from)?,
            datetime,
            fields,
            sortby: sortby.map(|s| s.into().into_iter().map(|s| s.parse().unwrap()).collect()),
            filter: filter.map(|f| f.into()).transpose()?,
            query,
            ..Default::default()
        },
    };
    if use_duckdb
        .unwrap_or_else(|| matches!(Format::infer_from_href(&href), Some(Format::Geoparquet(_))))
    {
        if let Some(max_items) = max_items {
            search.items.limit = Some(max_items.try_into()?);
        }
        let client = Client::from_href(href).map_err(Error::from)?;
        client
            .search_to_json(search)
            .map(|item_collection| item_collection.items)
            .map_err(Error::from)
            .map_err(PyErr::from)
    } else {
        let client = BlockingClient::new(&href).map_err(Error::from)?;
        let items = client.search(search).map_err(Error::from)?;
        if let Some(max_items) = max_items {
            items
                .take(max_items)
                .collect::<Result<_, _>>()
                .map_err(Error::from)
                .map_err(PyErr::from)
        } else {
            items
                .collect::<Result<_, _>>()
                .map_err(Error::from)
                .map_err(PyErr::from)
        }
    }
}

#[derive(FromPyObject)]
pub enum StringOrDict {
    String(String),
    Dict(Py<PyDict>),
}

#[derive(FromPyObject)]
pub enum StringOrList {
    String(String),
    List(Vec<String>),
}

impl StringOrDict {
    fn into<T: FromStr + DeserializeOwned>(self) -> PyResult<T>
    where
        Error: From<<T as FromStr>::Err>,
    {
        match self {
            Self::String(s) => s.parse().map_err(Error::from).map_err(PyErr::from),
            Self::Dict(dict) => {
                Python::with_gil(|py| pythonize::depythonize(dict.bind(py))).map_err(PyErr::from)
            }
        }
    }
}

impl StringOrList {
    fn into(self) -> Vec<String> {
        match self {
            Self::List(list) => list,
            Self::String(s) => vec![s],
        }
    }
}
