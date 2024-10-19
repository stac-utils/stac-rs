//! STAC command-line interface (CLI).
//!
//! # Examples
//!
//! ```no_run
//! use stac_cli::Args;
//! use clap::Parser;
//!
//! let args = Args::parse_from(["stacrs", "--help"]);
//! # tokio_test::block_on(async {
//! args.run().await.unwrap();
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

mod args;
mod error;
mod input;
mod options;
mod output;
#[cfg(feature = "python")]
mod python;
mod value;

pub use {args::Args, error::Error, value::Value};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(feature = "duckdb")]
use duckdb as _;
use tracing_subscriber as _;

#[cfg(test)]
use {assert_cmd as _, tokio_test as _};
