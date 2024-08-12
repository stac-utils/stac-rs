use crate::Output;
use thiserror::Error;

/// Crate specific error type.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// Custom error.
    #[error("{0}")]
    Custom(String),

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// [reqwest::Error]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [stac_api::Error]
    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// [stac_async::Error]
    #[error(transparent)]
    StacAsync(#[from] stac_async::Error),

    // /// [stac_geoparquet::Error]
    // #[error(transparent)]
    // StacGeoparquet(#[from] stac_geoparquet::Error),
    /// [stac_server::Error]
    #[error(transparent)]
    StacServer(#[from] stac_server::Error),

    /// [stac_validate::Error]
    #[error(transparent)]
    StacValidate(#[from] stac_validate::Error),

    /// [tokio::sync::mpsc::error::SendError]
    #[error(transparent)]
    SendOutput(#[from] tokio::sync::mpsc::error::SendError<Output>),

    /// [tokio::task::JoinError]
    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),

    /// Unsupported output format.
    #[error("unsupported output format: {0}")]
    UnsupportedFormat(String),

    /// Validation errors.
    #[error("validation errors: {0:?}")]
    Validation(Vec<serde_json::Value>),
}

impl Error {
    /// Returns this error return code.
    pub fn code(&self) -> i32 {
        // TODO make these codes more meaningful
        1
    }
}
