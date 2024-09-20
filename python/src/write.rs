use crate::{Error, Result};
use pyo3::{
    pyfunction,
    types::{PyAnyMethods, PyDict},
    Bound, PyAny, PyResult, Python,
};
use serde_json::Value;
use stac::{Format, Item, ItemCollection};
use tokio::runtime::Builder;

/// Writes STAC to a href.
///
/// Args:
///     href (str): The href to write to
///     value (dict[str, Any] | list[dict[str, Any]]): The value to write. This
///         can be a STAC dictionary or a list of items.
///     format (str | None): The output format to write. If not provided, will be
///         inferred from the href's extension.
///     options (list[tuple[str, str]] | None): Options for configuring an
///         object store, e.g. your AWS credentials.
///
/// Returns:
///     dict[str, str] | None: The result of putting data into an object store,
///         e.g. the e_tag and the version. None is returned if the file was written
///         locally.
///
/// Examples:
///     >>> with open("simple-item.json") as f:
///     ...     item = json.load(f)
///     >>> stacrs.write("out.parquet", [item])
#[pyfunction]
#[pyo3(signature = (href, value, *, format=None, options=None))]
pub fn write<'py>(
    py: Python<'py>,
    href: String,
    value: Bound<'_, PyAny>,
    format: Option<String>,
    options: Option<Vec<(String, String)>>,
) -> PyResult<Option<Bound<'py, PyDict>>> {
    let value: Value = pythonize::depythonize(&value)?;
    let value = if let Value::Array(array) = value {
        let items = array
            .into_iter()
            .map(|value| serde_json::from_value::<Item>(value).map_err(Error::from))
            .collect::<Result<Vec<_>>>()?;
        stac::Value::ItemCollection(ItemCollection::from(items))
    } else {
        serde_json::from_value(value).map_err(Error::from)?
    };
    let format = format
        .and_then(|f| f.parse::<Format>().ok())
        .or_else(|| Format::infer_from_href(&href))
        .unwrap_or_default();
    let runtime = Builder::new_current_thread().enable_all().build()?;
    let put_result = runtime
        .block_on(async move {
            format
                .put_opts(href, value, options.unwrap_or_default())
                .await
        })
        .map_err(Error::from)?;
    if let Some(put_result) = put_result {
        let dict = PyDict::new_bound(py);
        if let Some(e_tag) = put_result.e_tag {
            dict.set_item("e_tag", e_tag)?;
        }
        if let Some(version) = put_result.version {
            dict.set_item("version", version)?;
        }
        Ok(Some(dict))
    } else {
        Ok(None)
    }
}
