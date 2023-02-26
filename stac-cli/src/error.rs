use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    StacAsync(#[from] stac_async::Error),
}

impl Error {
    pub fn return_code(&self) -> i32 {
        unimplemented!()
    }
}
