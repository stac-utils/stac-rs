use stac::Value;
use url::Url;

/// Crate-specific error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Cannot download assets for the given value.
    #[error("cannot download")]
    CannotDownload(Value),

    /// [reqwest::header::InvalidHeaderName]
    #[error(transparent)]
    InvalidHeaderName(#[from] reqwest::header::InvalidHeaderName),

    /// [reqwest::header::InvalidHeaderValue]
    #[error(transparent)]
    InvalidHeaderValue(#[from] reqwest::header::InvalidHeaderValue),

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// [tokio::task::JoinError]
    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    /// [http::method::InvalidMethod]
    #[error(transparent)]
    HttpInvalidMethod(#[from] http::method::InvalidMethod),

    /// [reqwest::Error]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// The endpoint was not found.
    #[error("not found: {0}")]
    NotFound(Url),

    /// [tokio::sync::mpsc::error::SendError] for [crate::download::Message].
    #[error(transparent)]
    TokioMpscSendDownloadMessageError(
        #[from] tokio::sync::mpsc::error::SendError<crate::download::Message>,
    ),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// [stac_api::Error]
    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}
