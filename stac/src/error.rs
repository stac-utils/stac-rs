use crate::Value;
use serde_json::Value as JsonValue;
use thiserror::Error;

/// Error enum for crate-specific errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// [chrono::ParseError]
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    /// [geojson::Error]
    #[cfg(feature = "geo")]
    #[error(transparent)]
    Geojson(#[from] geojson::Error),

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Returned when the `type` field of a STAC object is not a [String].
    #[error("invalid \"type\" field: {0}")]
    InvalidTypeField(JsonValue),

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

    /// Returned when the `type` field of a STAC object does not equal `"Feature"`, `"Catalog"`, or `"Collection"`.
    #[error("unknown \"type\": {0}")]
    UnknownType(String),

    /// [url::ParseError]
    #[error(transparent)]
    Url(#[from] url::ParseError),

    /// [jsonschema::ValidationError], but owned.
    #[cfg(feature = "jsonschema")]
    #[error(transparent)]
    ValidationError(#[from] jsonschema::ValidationError<'static>),
}
