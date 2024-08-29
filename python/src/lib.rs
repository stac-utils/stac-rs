use pyo3::{create_exception, exceptions::PyException, prelude::*, types::PyDict};
use stac::Value;
use stac_validate::Validate;

create_exception!(stacrs, StacrsError, PyException, "An error in stacrs");

/// Validates a single href with json-schema.
///
/// Args:
///     href (str): The href of the STAC value to validate
///
/// Raises:
///     StacrsError: On any input/output error, or on a validation error
///
/// Examples:
///     >>> stacrs.validate_href("examples/simple-item.json")
///     >>> stacrs.validate_href("data/invalid-item.json")
///     Traceback (most recent call last):
///     File "<stdin>", line 1, in <module>
///     stacrs.StacrsError: Validation errors: "collection" is a required property
#[pyfunction]
fn validate_href(href: &str) -> PyResult<()> {
    let value: Value = stac::read(href).map_err(|err| StacrsError::new_err(err.to_string()))?;
    validate_value(value)
}

/// Validates a STAC dictionary with json-schema.
///
/// Args:
///     value (dict[str, Any]): The STAC value to validate
///
/// Raises:
///     StacrsError: On a validation error
///
/// Examples:
///     >>> with open("examples/simple-item.json") as f:
///     >>>     data = json.load(f)
///     >>> stacrs.validate(data)
#[pyfunction]
fn validate(value: &Bound<'_, PyDict>) -> PyResult<()> {
    let value: Value = pythonize::depythonize(value)?;
    validate_value(value)
}

fn validate_value(value: Value) -> PyResult<()> {
    if let Err(error) = value.validate() {
        match error {
            stac_validate::Error::Validation(errors) => {
                let mut message = "Validation errors: ".to_string();
                for error in errors {
                    message.push_str(&format!("{}, ", error));
                }
                message.pop();
                message.pop();
                Err(StacrsError::new_err(message))
            }
            _ => Err(StacrsError::new_err(error.to_string())),
        }
    } else {
        Ok(())
    }
}

/// A collection of functions for working with STAC, using Rust under the hood.
#[pymodule]
fn stacrs(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(validate_href, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add("StacrsError", m.py().get_type_bound::<StacrsError>())?;
    Ok(())
}
