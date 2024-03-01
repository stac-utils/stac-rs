use crate::Item;
use pyo3::prelude::*;

/// pystac-rs is a library for working with the SpatioTemporal Asset Catalog
/// (STAC) specification, written in Rust.
#[pymodule]
fn pystac_rs(_py: Python, m: &PyModule) -> PyResult<()> {
    m.add_class::<Item>()?;
    Ok(())
}
