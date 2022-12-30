use crate::Value;
use serde_json::Value as JsonValue;
use thiserror::Error;

/// Error enum for crate-specific errors.
#[derive(Error, Debug)]
pub enum Error {
    /// [std::io::Error]
    #[error("std::io error: {0}")]
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
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// [serde_json::Error]
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Returned when the `type` field of a STAC object does not equal `"Feature"`, `"Catalog"`, or `"Collection"`.
    #[error("unknown \"type\": {0}")]
    UnknownType(String),

    /// [url::ParseError]
    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),

    /// [jsonschema::ValidationError], but owned.
    #[cfg(feature = "jsonschema")]
    #[error(transparent)]
    ValidationError(#[from] jsonschema::ValidationError<'static>),
}
