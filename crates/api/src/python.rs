//! Functions for convert [pyo3] objects into **stac-api** structures.

use crate::{Error, Fields, Filter, Items, Search, Sortby};
use geojson::Geometry;
use pyo3::{
    exceptions::{PyException, PyValueError},
    types::PyDict,
    Bound, FromPyObject, PyErr, PyResult,
};
use stac::Bbox;

/// Creates a [Search] from Python arguments.
#[allow(clippy::too_many_arguments)]
pub fn search<'py>(
    intersects: Option<StringOrDict<'py>>,
    ids: Option<StringOrList>,
    collections: Option<StringOrList>,
    limit: Option<u64>,
    bbox: Option<Vec<f64>>,
    datetime: Option<String>,
    include: Option<StringOrList>,
    exclude: Option<StringOrList>,
    sortby: Option<StringOrList>,
    filter: Option<StringOrDict<'py>>,
    query: Option<Bound<'py, PyDict>>,
) -> PyResult<Search> {
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
    let query = query
        .map(|query| pythonize::depythonize(&query))
        .transpose()?;
    let bbox = bbox.map(Bbox::try_from).transpose().map_err(Error::from)?;
    let sortby = sortby.map(|sortby| {
        Vec::<String>::from(sortby)
            .into_iter()
            .map(|s| s.parse::<Sortby>().unwrap()) // the parse is infallible
            .collect::<Vec<_>>()
    });
    let filter = filter
        .map(|filter| match filter {
            StringOrDict::Dict(cql_json) => pythonize::depythonize(&cql_json).map(Filter::Cql2Json),
            StringOrDict::String(cql2_text) => Ok(Filter::Cql2Text(cql2_text)),
        })
        .transpose()?;
    let filter = filter
        .map(|filter| filter.into_cql2_json())
        .transpose()
        .map_err(Error::from)?;
    let items = Items {
        limit,
        bbox,
        datetime,
        query,
        fields,
        sortby,
        filter,
        ..Default::default()
    };

    let intersects = intersects
        .map(|intersects| match intersects {
            StringOrDict::Dict(json) => pythonize::depythonize(&json)
                .map_err(PyErr::from)
                .and_then(|json| {
                    Geometry::from_json_object(json)
                        .map_err(|err| PyValueError::new_err(err.to_string()))
                }),
            StringOrDict::String(s) => s
                .parse::<Geometry>()
                .map_err(|err| PyValueError::new_err(err.to_string())),
        })
        .transpose()?;
    let ids = ids.map(|ids| ids.into());
    let collections = collections.map(|ids| ids.into());
    Ok(Search {
        items,
        intersects,
        ids,
        collections,
    })
}

/// A string or dictionary.
///
/// Used for the CQL2 filter argument and for intersects.
#[derive(Debug, FromPyObject)]
pub enum StringOrDict<'py> {
    /// Text
    String(String),

    /// Json
    Dict(Bound<'py, PyDict>),
}

/// A string or a list.
///
/// Used for collections, ids, etc.
#[derive(Debug, FromPyObject)]
pub enum StringOrList {
    /// A string.
    String(String),

    /// A list.
    List(Vec<String>),
}

impl From<StringOrList> for Vec<String> {
    fn from(value: StringOrList) -> Vec<String> {
        match value {
            StringOrList::List(list) => list,
            StringOrList::String(s) => vec![s],
        }
    }
}

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        PyException::new_err(value.to_string())
    }
}
