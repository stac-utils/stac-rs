use serde_json::{Map, Value};
use thiserror::Error;

/// Crate-specific error enum.
#[derive(Debug, Error)]
pub enum Error {
    /// Queries cannot be converted to strings.
    #[error("cannot convert queries to strings")]
    CannotConvertQueryToString(Map<String, Value>),

    /// CQL2 JSON cannot (currently) be converted to strings.
    ///
    /// TODO support conversion
    #[error("cannot convert cql2-json to strings")]
    CannotConvertCql2JsonToString(Map<String, Value>),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [serde_urlencoded::ser::Error]
    #[error(transparent)]
    SerdeUrlencodedSer(#[from] serde_urlencoded::ser::Error),

    /// [std::num::TryFromIntError]
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
