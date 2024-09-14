use crate::Error;
use pyo3::{prelude::*, types::PyDict};
use stac::{Migrate, Value};

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
pub fn migrate<'py>(
    value: &Bound<'py, PyDict>,
    version: Option<&str>,
) -> PyResult<Bound<'py, PyDict>> {
    let py = value.py();
    let value: Value = pythonize::depythonize(value)?;
    let version = version
        .map(|version| version.parse())
        .transpose()
        .map_err(Error::from)?
        .unwrap_or_default();
    let value = value.migrate(&version).map_err(Error::from)?;
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
pub fn migrate_href<'py>(
    py: Python<'py>,
    href: &str,
    version: Option<&str>,
) -> PyResult<Bound<'py, PyDict>> {
    let value: Value = stac::read(href).map_err(Error::from)?;
    let version = version
        .map(|version| version.parse())
        .transpose()
        .map_err(Error::from)?
        .unwrap_or_default();
    let value = value.migrate(&version).map_err(Error::from)?;
    let value = pythonize::pythonize(py, &value)?;
    value.downcast_into().map_err(PyErr::from)
}
