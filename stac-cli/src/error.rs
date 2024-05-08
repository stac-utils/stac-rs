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

    /// [stac_server::Error]
    #[error(transparent)]
    StacServer(#[from] stac_server::Error),

    /// [stac_validate::Error]
    #[error(transparent)]
    StacValidate(#[from] stac_validate::Error),

    /// [tokio::task::JoinError]
    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),
}

impl Error {
    /// Returns this error return code.
    pub fn code(&self) -> i32 {
        // TODO make these codes more meaningful
        1
    }
}
