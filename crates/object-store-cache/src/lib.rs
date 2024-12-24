//! Work with [ObjectStore](object_store::ObjectStore) in STAC.
//!
//! Features:
//!  - cache used objects_stores based on url and options
//!  - read cloud creadentials from env
//!

mod cache;
mod error;

pub use cache::parse_url_opts;

pub use error::Error;

/// Custom [Result](std::result::Result) type for this crate.
pub type Result<T> = std::result::Result<T, Error>;
