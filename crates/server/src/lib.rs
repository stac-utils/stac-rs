//! A [STAC API](https://github.com/radiantearth/stac-api-spec) server written in Rust.

#![deny(
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    warnings
)]

mod api;
mod backend;
mod error;
#[cfg(feature = "axum")]
pub mod routes;

pub use api::Api;
#[cfg(feature = "pgstac")]
pub use backend::PgstacBackend;
pub use backend::{Backend, MemoryBackend};
pub use error::Error;

/// A crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

/// The default catalog id.
pub const DEFAULT_ID: &str = "stac-server-rs";

/// The default catalog description.
pub const DEFAULT_DESCRIPTION: &str = "A STAC API server written in Rust";

/// The default limit.
pub const DEFAULT_LIMIT: u64 = 10;

#[cfg(test)]
use tokio_test as _;

#[cfg(all(test, not(feature = "axum")))]
use tower as _;
