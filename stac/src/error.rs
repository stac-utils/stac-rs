use crate::{Value, Version};
use serde_json::Value as JsonValue;
use thiserror::Error;

/// Error enum for crate-specific errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// [chrono::ParseError]
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    /// [gdal::errors::GdalError]
    #[cfg(feature = "gdal")]
    #[error(transparent)]
    GdalError(#[from] gdal::errors::GdalError),

    /// GDAL is not enabled.
    #[error("gdal is not enabled")]
    GdalNotEnabled,

    /// [geojson::Error]
    #[error(transparent)]
    Geojson(#[from] geojson::Error),

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Returned when the `type` field of a STAC object is not a [String].
    #[error("invalid \"type\" field: {0}")]
    InvalidTypeField(JsonValue),

    /// Returned when a property name conflicts with a top-level STAC field, or
    /// it's an invalid top-level field name.
    #[error("invalid attribute name: {0}")]
    InvalidAttribute(String),

    /// Returned when a STAC object has the wrong type field.
    #[error("incorrect type: expected={expected}, actual={actual}")]
    IncorrectType {
        /// The actual type field on the object.
        actual: String,
        /// The expected value.
        expected: String,
    },

    /// This vector is not a valid bounding box.
    #[error("invalid bbox: {0:?}")]
    InvalidBbox(Vec<f64>),

    /// This string is not a valid datetime interval.
    #[error("invalid datetime: {0}")]
    InvalidDatetime(String),

    /// Returned when there is not a `id` field on a STAC object
    #[error("no \"id\" field in the JSON object")]
    MissingId,

    /// Returned when there is not a `type` field on a STAC object
    #[error("no \"type\" field in the JSON object")]
    MissingType,

    /// Returned when an object is expected to have an href, but it doesn't.
    #[error("object has no href")]
    MissingHref,

    /// This value is not an item.
    #[error("value is not an item")]
    NotAnItem(Value),

    /// This value is not a catalog.
    #[error("value is not a catalog")]
    NotACatalog(Value),

    /// This value is not a collection.
    #[error("value is not a collection")]
    NotACollection(Value),

    /// This value is not an object.
    #[error("not an object")]
    NotAnObject(serde_json::Value),

    /// Returned when trying to read from a url but the `reqwest` feature is not enabled.
    #[error("reqwest is not enabled")]
    ReqwestNotEnabled,

    /// [reqwest::Error]
    #[cfg(feature = "reqwest")]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [std::num::TryFromIntError]
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),

    /// Returned when the `type` field of a STAC object does not equal `"Feature"`, `"Catalog"`, or `"Collection"`.
    #[error("unknown \"type\": {0}")]
    UnknownType(String),

    /// Unsupported version.
    #[error("unsupported version: {0}")]
    UnsupportedVersion(String),

    /// Unsupported migration.
    #[error("unsupported migration: {0} to {1}")]
    UnsupportedMigration(Version, Version),

    /// [url::ParseError]
    #[error(transparent)]
    Url(#[from] url::ParseError),
}
