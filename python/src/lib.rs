#![deny(unused_crate_dependencies)]

use pyo3::{create_exception, exceptions::PyException, prelude::*, types::PyDict};
use stac::{Migrate, Value};
use stac_validate::Validate;

create_exception!(stacrs, StacrsError, PyException, "An error in stacrs");

/// Migrates a STAC dictionary to another version.
///
/// Migration can be as simple as updating the `stac_version` attribute, but
/// sometimes can be more complicated. For example, when migrating to v1.1.0,
/// [eo:bands and raster:bands should be consolidated to the new bands
/// structure](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0-beta.1).
///
/// See [the stac-rs
/// documentation](https://docs.rs/stac/latest/stac/enum.Version.html) for
/// supported versions.
///
/// Args:
///     value (dict[str, Any]): The STAC value to migrate
///     version (str | None): The version to migrate to. If not provided, the
///         value will be migrated to the latest stable version.
///
/// Returns:
///     dict[str, Any]: The migrated dictionary
///
/// Examples:
///     >>> with open("examples/simple-item.json") as f:
///     >>>     item = json.load(f)
///     >>> item = stacrs.migrate(item, "1.1.0-beta.1")
///     >>> assert item["stac_version"] == "1.1.0-beta.1"
#[pyfunction]
#[pyo3(signature = (value, version=None))]
fn migrate<'py>(value: &Bound<'py, PyDict>, version: Option<&str>) -> PyResult<Bound<'py, PyDict>> {
    let py = value.py();
    let value: Value = pythonize::depythonize(value)?;
    let version = version
        .map(|version| version.parse().unwrap())
        .unwrap_or_default();
    let value = value
        .migrate(&version)
        .map_err(|err| StacrsError::new_err(err.to_string()))?;
    let value = pythonize::pythonize(py, &value)?;
    value.downcast_into().map_err(PyErr::from)
}

/// Migrates a STAC dictionary at the given href to another version.
///
/// Migration can be as simple as updating the `stac_version` attribute, but
/// sometimes can be more complicated. For example, when migrating to v1.1.0,
/// [eo:bands and raster:bands should be consolidated to the new bands
/// structure](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0-beta.1).
///
/// See [the stac-rs
/// documentation](https://docs.rs/stac/latest/stac/enum.Version.html) for
/// supported versions.
///
/// Args:
///     href (str): The href to read the STAC object from
///     version (str | None): The version to migrate to. If not provided, the
///         value will be migrated to the latest stable version.
///
/// Examples:
///     >>> item = stacrs.migrate_href("examples/simple-item.json", "1.1.0-beta.1")
///     >>> assert item["stac_version"] == "1.1.0-beta.1"
#[pyfunction]
#[pyo3(signature = (href, version=None))]
fn migrate_href<'py>(
    py: Python<'py>,
    href: &str,
    version: Option<&str>,
) -> PyResult<Bound<'py, PyDict>> {
    let value: Value = stac::read(href).map_err(|err| StacrsError::new_err(err.to_string()))?;
    let version = version
        .map(|version| version.parse().unwrap())
        .unwrap_or_default();
    let value = value
        .migrate(&version)
        .map_err(|err| StacrsError::new_err(err.to_string()))?;
    let value = pythonize::pythonize(py, &value)?;
    value.downcast_into().map_err(PyErr::from)
}

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
    m.add_function(wrap_pyfunction!(migrate, m)?)?;
    m.add_function(wrap_pyfunction!(migrate_href, m)?)?;
    m.add_function(wrap_pyfunction!(validate_href, m)?)?;
    m.add_function(wrap_pyfunction!(validate, m)?)?;
    m.add("StacrsError", m.py().get_type_bound::<StacrsError>())?;
    Ok(())
}
