//! Read and write STAC to/from [geoarrow](https://github.com/geoarrow/geoarrow).
//!
//! The arrow data formatted per the [stac-geoparquet
//! spec](https://github.com/stac-utils/stac-geoparquet/blob/main/spec/stac-geoparquet-spec.md).
//!
//! # Reading
//!
//! Use top-level function [geo_table_to_items] to convert [geoarrow] record
//! batches to a vector of [Items](stac::Item).
//!
//! ```
//! use std::fs::File;
//!
//! let file = File::open("data/naip.parquet").unwrap();
//! let geo_table = geoarrow::io::parquet::read_geoparquet(file, Default::default()).unwrap();
//! let items = stac_arrow::geo_table_to_items(geo_table).unwrap();
//! assert_eq!(items.len(), 5);
//! ```
//!
//! The [Reader] structure provides more control over the process.
//!
//! # Writing
//!
//! For writing, there is a top level [items_to_geo_table]:
//!
//! ```
//! use stac::ItemCollection;
//!
//! let item_collection: ItemCollection = stac::read_json("data/naip.json").unwrap();
//! let geo_table = stac_arrow::items_to_geo_table(item_collection.items).unwrap();
//! ```
//!
//! The [Writer] structure provides more control.
//!
//! # IO
//!
//! This library does not provide any IO (reading or writing to/from disk or the network) — use [geoarrow::io].

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
    unused_results,
    warnings
)]

mod error;
mod read;
mod write;

pub use {
    error::Error,
    read::{geo_table_to_items, Reader},
    write::{items_to_geo_table, Writer},
};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

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

#[cfg(test)]
use criterion as _;
