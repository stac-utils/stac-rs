mod args;
mod error;
mod subcommand;

pub use {args::Args, error::Error, subcommand::Subcommand};

pub type Result<T> = std::result::Result<T, Error>;
