mod args;
mod error;

pub use {
    args::{Args, Command},
    error::Error,
};

pub type Result<T> = std::result::Result<T, Error>;
