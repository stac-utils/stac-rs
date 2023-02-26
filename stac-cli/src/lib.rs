mod args;
mod download;
mod error;

pub use {
    args::{Args, Command},
    download::download,
    error::Error,
};

pub type Result<T> = std::result::Result<T, Error>;
