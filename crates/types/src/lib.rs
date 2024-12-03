mod error;
mod fields;
mod href;
pub mod link;
mod migrate;
pub mod mime;
mod version;

pub type Result<T> = std::result::Result<T, Error>;

pub use {
    error::Error,
    fields::Fields,
    href::{Href, RealizedHref, SelfHref},
    link::{Link, Links},
    migrate::Migrate,
    version::Version,
};

/// The default STAC version of this library.
pub const STAC_VERSION: Version = Version::v1_1_0;
extern crate self as stac_types;
