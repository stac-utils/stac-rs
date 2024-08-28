use pyo3::{create_exception, exceptions::PyException, prelude::*};
use stac::Value;
use stac_validate::Validate;

create_exception!(stacrs, StacrsError, PyException, "An error in stacrs");

/// Validates a single href with json-schema.
#[pyfunction]
fn validate_href(href: &str) -> PyResult<()> {
    let value: Value = stac::read(href).map_err(|err| StacrsError::new_err(err.to_string()))?;
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
    m.add("StacrsError", m.py().get_type_bound::<StacrsError>())?;
    Ok(())
}
