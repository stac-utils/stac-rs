//! Asynchronous I/O for [STAC](https://stacspec.org/), built on [stac-rs](https://github.com/gadomski/stac-rs)
//!
//! This is just a small library that provides an async version of [stac::read].
//! It also includes a thin wrapper around [reqwest::Client] for efficiently
//! getting multiple STAC items in the same session.
//!
//! # Examples
//!
//! ```
//! # tokio_test::block_on(async {
//! let item: stac::Item = stac_async::read("data/simple-item.json").await.unwrap();
//! # })
//! ```

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
    pointer_structural_match,
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
    unused_results
)]

mod api_client;
mod client;
pub mod download;
mod error;
mod io;

pub use {
    api_client::ApiClient,
    client::Client,
    download::{download, Download, Downloader},
    error::Error,
    io::{read, read_json, write_json_to_path},
};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
use tokio_test as _; // used for doc tests
