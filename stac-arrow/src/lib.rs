//! Read and write STAC to/from [geoarrow](https://github.com/geoarrow/geoarrow).
//!
//! The arrow data formatted per the [stac-geoparquet
//! spec](https://github.com/stac-utils/stac-geoparquet/blob/main/spec/stac-geoparquet-spec.md).

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
    read::{record_batch_to_items, Reader},
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
