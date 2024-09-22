use crate::Error;
use pyo3::{
    prelude::*,
    types::{PyDict, PyList},
};
use serde::de::DeserializeOwned;
use stac::Format;
use stac_api::{BlockingClient, Fields, Item, ItemCollection, Items, Search};
use std::str::FromStr;
use tokio::runtime::Builder;

/// Searches a STAC API server.
///
/// Args:
///     href (str): The STAC API to search.
///     intersects (str | dict[str, Any] | GeoInterface | None): Searches items
///         by performing intersection between their geometry and provided GeoJSON
///         geometry.
///     ids (list[str] | None): Array of Item ids to return.
///     collections (list[str] | None): Array of one or more Collection IDs that
///         each matching Item must be in.
///     max_items (int | None): The maximum number of items to iterate through.
///     limit (int | None): The page size returned from the server. Use
///         `max_items` to actually limit the number of items returned from this
///         function.
///     bbox (list[float] | None): Requested bounding box.
///     datetime (str | None): Single date+time, or a range (‘/’ separator),
///         formatted to RFC 3339, section 5.6.  Use double dots .. for open
///         date ranges.
///     include (list[str]] | None): fields to include in the response (see [the
///         extension
///         docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
///         for more on the semantics).
///     exclude (list[str]] | None): fields to exclude from the response (see [the
///         extension
///         docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
///         for more on the semantics).
///     sortby (list[str] | None): Fields by which to sort results (use `-field` to sort descending).
///     filter (str | dict[str, Any] | none): CQL2 filter expression. Strings
///         will be interpreted as cql2-text, dictionaries as cql2-json.
///     query (dict[str, Any] | None): Additional filtering based on properties.
///         It is recommended to use filter instead, if possible.
///
/// Returns:
///     list[dict[str, Any]]: A list of the returned STAC items.
///
/// Examples:
///     >>> items = stacrs.search(
///     ...     "https://landsatlook.usgs.gov/stac-server",
///     ...     collections=["landsat-c2l2-sr"],
///     ...     intersects={"type": "Point", "coordinates": [-105.119, 40.173]},
///     ...     sortby="-properties.datetime",
///     ...     max_items=1,
///     ... )
#[pyfunction]
#[pyo3(signature = (href, *, intersects=None, ids=None, collections=None, max_items=None, limit=None, bbox=None, datetime=None, include=None, exclude=None, sortby=None, filter=None, query=None))]
pub fn search<'py>(
    py: Python<'py>,
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
) -> PyResult<Bound<'py, PyList>> {
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
    )?;
    pythonize::pythonize(py, &items)
        .map_err(PyErr::from)
        .and_then(|v| v.extract())
}

/// Searches a STAC API server and saves the result to an output file.
///
/// Args:
///     outfile (str): The output href. This can be a local file path, or any
///         url scheme supported by [stac::object_store::write].
///     href (str): The STAC API to search.
///     intersects (str | dict[str, Any] | GeoInterface | None): Searches items
///         by performing intersection between their geometry and provided GeoJSON
///         geometry.
///     ids (list[str] | None): Array of Item ids to return.
///     collections (list[str] | None): Array of one or more Collection IDs that
///         each matching Item must be in.
///     max_items (int | None): The maximum number of items to iterate through.
///     limit (int | None): The page size returned from the server. Use
///         `max_items` to actually limit the number of items returned from this
///         function.
///     bbox (list[float] | None): Requested bounding box.
///     datetime (str | None): Single date+time, or a range (‘/’ separator),
///         formatted to RFC 3339, section 5.6.  Use double dots .. for open
///         date ranges.
///     include (list[str]] | None): fields to include in the response (see [the
///         extension
///         docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
///         for more on the semantics).
///     exclude (list[str]] | None): fields to exclude from the response (see [the
///         extension
///         docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
///         for more on the semantics).
///     sortby (list[str] | None): Fields by which to sort results (use `-field` to sort descending).
///     filter (str | dict[str, Any] | none): CQL2 filter expression. Strings
///         will be interpreted as cql2-text, dictionaries as cql2-json.
///     query (dict[str, Any] | None): Additional filtering based on properties.
///         It is recommended to use filter instead, if possible.
///     format (str | None): The output format. If none, will be inferred from
///         the outfile extension, and if that fails will fall back to compact JSON.
///     options (list[tuple[str, str]] | None): Configuration values to pass to the object store backend.
///
/// Returns:
///     list[dict[str, Any]]: A list of the returned STAC items.
///
/// Examples:
///     >>> items = stacrs.search_to("out.parquet",
///     ...     "https://landsatlook.usgs.gov/stac-server",
///     ...     collections=["landsat-c2l2-sr"],
///     ...     intersects={"type": "Point", "coordinates": [-105.119, 40.173]},
///     ...     sortby="-properties.datetime",
///     ...     max_items=1,
///     ... )
#[pyfunction]
#[pyo3(signature = (outfile, href, *, intersects=None, ids=None, collections=None, max_items=None, limit=None, bbox=None, datetime=None, include=None, exclude=None, sortby=None, filter=None, query=None, format=None, options=None))]
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
) -> PyResult<Vec<Item>> {
    let client = BlockingClient::new(&href).map_err(Error::from)?;
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
    let search = Search {
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
                Python::with_gil(|py| pythonize::depythonize(&dict.bind(py))).map_err(PyErr::from)
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
