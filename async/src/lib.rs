//! Asynchronous I/O for [STAC](https://stacspec.org/), built on [stac-rs](https://github.com/stac-utils/stac-rs)
//!
//! # Examples
//!
//! Read a single STAC item:
//!
//! ```
//! # tokio_test::block_on(async {
//! let item: stac::Item = stac_async::read("examples/simple-item.json").await.unwrap();
//! # })
//! ```
//!
//! Build an API client to read from a [STAC API](https://github.com/radiantearth/stac-api-spec):
//!
//! ```no_run
//! use futures_util::StreamExt;
//! let url = "https://planetary-computer.microsoft.com/api/stac/v1";
//! let api_client = stac_async::ApiClient::new(url).unwrap();
//! # tokio_test::block_on(async {
//! // Get the first sentinel2 item using an asynchronous stream
//! let items = api_client.items("sentinel2-l2a", None).await.unwrap();
//! tokio::pin!(items);
//! let item = items.next().await.unwrap().unwrap();
//! # })
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
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

mod api_client;
mod client;
mod error;
mod io;

pub use {
    api_client::ApiClient,
    client::Client,
    error::Error,
    io::{read, read_json, write_json_to_path},
};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
use tokio_test as _; // used for doc tests

// From https://github.com/rust-lang/cargo/issues/383#issuecomment-720873790,
// may they be forever blessed.
#[cfg(doctest)]
mod readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}
