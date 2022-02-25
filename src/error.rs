use crate::stac::Handle;
use serde_json::Value;
use thiserror::Error;
use url::Url;

/// Error enum for crate-specific errors.
#[derive(Error, Debug)]
pub enum Error {
    /// Returned when trying to write urls from the default writer.
    #[error("cannot write url: {0}")]
    CannotWriteUrl(Url),

    /// [std::io::Error]
    #[error("std::io error: {0}")]
    Io(#[from] std::io::Error),

    /// [std::convert::Infallible]
    #[error("std::conver::Infallible: {0}")]
    Infallible(#[from] std::convert::Infallible),

    /// Returned when trying to access data in a [Stac](crate::Stac) with an invalid [Handle].
    #[error("invalid handle: {0:?}")]
    InvalidHandle(Handle),

    /// Returned when the `type` field of a STAC object is not a [String].
    #[error("invalid \"type\" field: {0}")]
    InvalidTypeField(Value),

    /// Returned when the `type` field of a STAC object does not equal `"Feature"`, `"Catalog"`, or `"Collection"`.
    #[error("invalid \"type\" value: {0}")]
    InvalidTypeValue(String),

    /// Returned when there is not a `type` field on a STAC object
    #[error("no \"type\" field in the JSON object")]
    MissingType,

    /// Returned when trying to write an [Object](crate::Object) that does not have an href.
    #[error("object has no href, cannot write")]
    MissingHref,

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

    /// Mismatch between expected and actual type fields.
    #[error("type mismatch: expected={expected}, actual={actual}")]
    TypeMismatch {
        /// The expected type field.
        expected: String,
        /// The actual type field.
        actual: String,
    },

    /// [url::ParseError]
    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),
}
