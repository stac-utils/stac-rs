mod args;
mod command;
mod error;

pub use {args::Args, command::Command, error::Error};

pub type Result<T> = std::result::Result<T, Error>;
