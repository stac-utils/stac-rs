use thiserror::Error;

/// Crate-specific error enum.
#[derive(Debug, Error)]
pub enum Error {
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
