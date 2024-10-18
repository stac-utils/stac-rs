use crate::Error;
use pyo3::{
    pyfunction,
    types::{PyAnyMethods, PyDict},
    Bound, PyErr, PyResult, Python,
};
use stac::{Format, Value};
use tokio::runtime::Builder;

#[pyfunction]
#[pyo3(signature = (href, *, format=None, options=None))]
pub fn read<'py>(
    py: Python<'py>,
    href: String,
    format: Option<String>,
    options: Option<Vec<(String, String)>>,
) -> PyResult<Bound<'py, PyDict>> {
    let format = format
        .and_then(|f| f.parse::<Format>().ok())
        .or_else(|| Format::infer_from_href(&href))
        .unwrap_or_default();
    let options = options.unwrap_or_default();
    let runtime = Builder::new_current_thread().enable_all().build()?;
    let value = runtime
        .block_on(async move { format.get_opts::<Value, _, _, _>(href, options).await })
        .map_err(Error::from)?;
    pythonize::pythonize(py, &value)
        .map_err(PyErr::from)
        .and_then(|v| v.extract())
}
