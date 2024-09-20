use crate::{Error, Result};
use pyo3::{prelude::*, types::PyDict};
use stac::Value;
use stac_validate::ValidateBlocking;

/// Validates a single href with json-schema.
///
/// Args:
///     href (str): The href of the STAC value to validate
///
/// Raises:
///     Exception: On any input/output error, or on a validation error
///
/// Examples:
///     >>> stacrs.validate_href("examples/simple-item.json")
///     >>> stacrs.validate_href("data/invalid-item.json")
///     Traceback (most recent call last):
///     File "<stdin>", line 1, in <module>
///     Exception: Validation errors: "collection" is a required property
#[pyfunction]
pub fn validate_href(href: &str) -> Result<()> {
    let value: Value = stac::read(href)?;
    validate_value(value)
}

/// Validates a STAC dictionary with json-schema.
///
/// Args:
///     value (dict[str, Any]): The STAC value to validate
///
/// Raises:
///     Exception: On a validation error
///
/// Examples:
///     >>> with open("examples/simple-item.json") as f:
///     >>>     data = json.load(f)
///     >>> stacrs.validate(data)
#[pyfunction]
pub fn validate(value: &Bound<'_, PyDict>) -> PyResult<()> {
    let value: Value = pythonize::depythonize(value)?;
    validate_value(value).map_err(Error::into)
}

fn validate_value(value: Value) -> Result<()> {
    if let Err(error) = value.validate_blocking() {
        match error {
            stac_validate::Error::Validation(errors) => {
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
