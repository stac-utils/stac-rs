use crate::download::Progress;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Io(#[from] std::io::Error),

    #[error(transparent)]
    ProgressSend(#[from] tokio::sync::mpsc::error::SendError<Progress>),

    #[error(transparent)]
    StacAsync(#[from] stac_async::Error),

    #[error(transparent)]
    TokioJoinError(#[from] tokio::task::JoinError),
}

impl Error {
    pub fn return_code(&self) -> i32 {
        unimplemented!()
    }
}
