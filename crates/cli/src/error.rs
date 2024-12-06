use crate::Value;
use thiserror::Error;

/// Crate specific error type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// [object_store::Error]
    #[error(transparent)]
    ObjectStore(#[from] object_store::Error),

    /// [object_store::path::Error]
    #[error(transparent)]
    ObjectStorePath(#[from] object_store::path::Error),

    /// [reqwest::Error]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [stac_api::Error]
    #[error("[stac-api] {0}")]
    StacApi(#[from] stac_api::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// [stac_duckdb::Error]
    #[cfg(feature = "duckdb")]
    #[error(transparent)]
    StacDuckdb(#[from] stac_duckdb::Error),

    /// [stac_server::Error]
    #[error(transparent)]
    StacServer(#[from] stac_server::Error),

    /// [tokio::sync::mpsc::error::SendError]
    #[error(transparent)]
    TokioSend(#[from] tokio::sync::mpsc::error::SendError<Value>),

    /// [tokio::task::JoinError]
    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),

    /// [tokio_postgres::Error]
    #[cfg(feature = "pgstac")]
    #[error(transparent)]
    TokioPostgres(#[from] tokio_postgres::Error),

    /// [std::num::TryFromIntError]
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),

    /// Unsupported format.
    #[error("unsupported (or unknown) format: {0}")]
    UnsupportedFormat(String),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),
}

impl Error {
    /// Returns this error return code.
    pub fn code(&self) -> i32 {
        // TODO make these codes more meaningful
        1
    }
}
