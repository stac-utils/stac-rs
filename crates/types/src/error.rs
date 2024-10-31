use crate::Version;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    /// Returned when a STAC object has the wrong type field.
    #[error("incorrect type: expected={expected}, actual={actual}")]
    IncorrectType {
        /// The actual type field on the object.
        actual: String,

        /// The expected value.
        expected: String,
    },

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// There is not an href, when an href is required.
    #[error("no href")]
    NoHref,

    /// This is not a JSON object.
    #[error("json value is not an object")]
    NotAnObject(serde_json::Value),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// Unsupported migration.
    #[error("unsupported migration: {0} to {1}")]
    UnsupportedMigration(Version, Version),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
