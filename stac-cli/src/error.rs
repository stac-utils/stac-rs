use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("{0}")]
    Custom(String),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    #[error(transparent)]
    Stac(#[from] stac::Error),

    #[error(transparent)]
    StacAsync(#[from] stac_async::Error),

    #[error(transparent)]
    StacServer(#[from] stac_server::Error),

    #[error(transparent)]
    StacValidate(#[from] stac_validate::Error),

    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),
}

impl Error {
    pub fn code(&self) -> i32 {
        // TODO make these codes more meaningful
        1
    }
}
