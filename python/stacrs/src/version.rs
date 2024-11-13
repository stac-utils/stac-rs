use duckdb::Connection;
use pyo3::prelude::*;

#[pyfunction]
#[pyo3(signature = (name=None))]
pub fn version(name: Option<String>) -> Option<String> {
    if let Some(name) = name {
        if name.eq_ignore_ascii_case("stac") {
            Some(stac::version().to_string())
        } else if name.eq_ignore_ascii_case("stac-api") {
            Some(stac_api::version().to_string())
        } else if name.eq_ignore_ascii_case("stac-duckdb") {
            Some(stac_duckdb::version().to_string())
        } else if name.eq_ignore_ascii_case("duckdb") {
            Some(
                Connection::open_in_memory()
                    .and_then(|c| c.version().map(|s| s[1..].to_string()))
                    .unwrap_or("unknown".to_string()),
            )
        } else {
            None
        }
    } else {
        Some(env!("CARGO_PKG_VERSION").to_string())
    }
}
