use std::convert::Infallible;

use pyo3::{exceptions::PyException, PyErr};

pub struct Error(pub String);

impl From<stac::Error> for Error {
    fn from(value: stac::Error) -> Self {
        Error(value.to_string())
    }
}

impl From<stac_api::Error> for Error {
    fn from(value: stac_api::Error) -> Self {
        Error(value.to_string())
    }
}

impl From<stac_validate::Error> for Error {
    fn from(value: stac_validate::Error) -> Self {
        Error(value.to_string())
    }
}

impl From<geojson::Error> for Error {
    fn from(value: geojson::Error) -> Self {
        Error(value.to_string())
    }
}

impl From<serde_json::Error> for Error {
    fn from(value: serde_json::Error) -> Self {
        Error(value.to_string())
    }
}

impl From<Infallible> for Error {
    fn from(_: Infallible) -> Self {
        unreachable!()
    }
}

impl From<Error> for PyErr {
    fn from(value: Error) -> Self {
        PyException::new_err(value.0)
    }
}
