use stac::Value;
use thiserror::Error;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("invalid STAC")]
    InvalidValue(Value),

    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    #[error(transparent)]
    StacApi(#[from] stac_api::Error),

    #[error(transparent)]
    StacAsync(#[from] stac_async::Error),

    #[error(transparent)]
    StacValidate(#[from] stac_validate::Error),

    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),

    #[error("many validation errors")]
    ValidationGroup(Vec<stac_validate::Error>),
}

impl Error {
    pub fn return_code(&self) -> i32 {
        // TODO make these codes more meaningful
        1
    }
}
