use crate::download::Progress;
use stac::Value;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("unsupported STAC type for downloading assets")]
    CannotDownload(Value),

    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error("invalid STAC")]
    InvalidValue(Value),

    #[error(transparent)]
    ProgressSend(#[from] tokio::sync::mpsc::error::SendError<Progress>),

    #[error(transparent)]
    StacAsync(#[from] stac_async::Error),

    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),
}

impl Error {
    pub fn return_code(&self) -> i32 {
        // TODO make these codes more meaningful
        1
    }
}
