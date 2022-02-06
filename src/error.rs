use crate::stac::Handle;
use serde_json::Value;
use thiserror::Error;

/// Error enum for the stac crate.
#[derive(Error, Debug)]
pub enum Error {
    /// std::io::Error.
    #[error("std::io error: {0}")]
    Io(#[from] std::io::Error),

    /// An invalid handle for a `Stac`
    #[error("invalid handle: {0:?}")]
    InvalidHandle(Handle),

    /// Invalid type field (not a string).
    #[error("Invalid \"type\" field: {0}")]
    InvalidTypeField(Value),

    /// Invalid type value.
    #[error("Invalid \"type\" value: {0}")]
    InvalidTypeValue(String),

    /// No "type" field in the JSON object, unable to parse as STAC object.
    #[error("No \"type\" field in the JSON object, unable to parse as STAC object")]
    MissingType,

    /// Reqwest is not enabled, so we cannot read URLs with the default reader.
    #[error("Reqwest is not enabled, so we cannot read URLs with the default reader")]
    ReqwestNotEnabled,

    /// A reqwest error.
    #[cfg(feature = "reqwest")]
    #[error("reqwest error: {0}")]
    Reqwest(#[from] reqwest::Error),

    /// A serde_json error.
    #[error("serde_json error: {0}")]
    SerdeJson(#[from] serde_json::Error),

    /// Unresolvable node.
    #[error("the node is unresolved and does not have an href")]
    UnresolvableNode,

    /// A url parse error.
    #[error("url parse error: {0}")]
    Url(#[from] url::ParseError),
}
