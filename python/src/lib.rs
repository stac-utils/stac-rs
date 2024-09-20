#![deny(unused_crate_dependencies)]

mod error;
mod migrate;
mod read;
mod search;
mod validate;
mod write;

use error::Error;
use pyo3::prelude::*;

type Result<T> = std::result::Result<T, Error>;

/// A collection of functions for working with STAC, using Rust under the hood.
#[pymodule]
fn stacrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(migrate::migrate, m)?)?;
    m.add_function(wrap_pyfunction!(migrate::migrate_href, m)?)?;
    m.add_function(wrap_pyfunction!(read::read, m)?)?;
    m.add_function(wrap_pyfunction!(search::search, m)?)?;
    m.add_function(wrap_pyfunction!(search::search_to, m)?)?;
    m.add_function(wrap_pyfunction!(validate::validate, m)?)?;
    m.add_function(wrap_pyfunction!(validate::validate_href, m)?)?;
    m.add_function(wrap_pyfunction!(write::write, m)?)?;
    Ok(())
}
