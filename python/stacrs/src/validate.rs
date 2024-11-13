use crate::{Error, Result};
use pyo3::{prelude::*, types::PyDict};
use stac::{ValidateBlocking, Value};

#[pyfunction]
pub fn validate_href(href: &str) -> Result<()> {
    let value: Value = stac::read(href)?;
    validate_value(value)
}

#[pyfunction]
pub fn validate(value: &Bound<'_, PyDict>) -> PyResult<()> {
    let value: Value = pythonize::depythonize(value)?;
    validate_value(value).map_err(Error::into)
}

fn validate_value(value: Value) -> Result<()> {
    if let Err(error) = value.validate_blocking() {
        match error {
            stac::Error::Validation(errors) => {
                let mut message = "Validation errors: ".to_string();
                for error in errors {
                    message.push_str(&format!("{}, ", error));
                }
                message.pop();
                message.pop();
                Err(Error(message))
            }
            _ => Err(error.into()),
        }
    } else {
        Ok(())
    }
}
