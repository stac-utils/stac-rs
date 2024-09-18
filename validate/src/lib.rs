//! Validate STAC objects with [json-schema](https://json-schema.org/).
//!
//! # Examples
//!
//! Validation is provided via the [Validate] trait:
//!
//! ```
//! use stac::Item;
//! use stac_validate::Validate;
//!
//! # tokio_test::block_on(async {
//! Item::new("an-id").validate().await.unwrap();
//! # })
//! ```
//!
//! If you're working in a blocking context (not async), enable the `blocking` feature and use [ValidateBlocking]:
//!
//! ```
//! # use stac::Item;
//! #[cfg(feature = "blocking")]
//! {
//!     use stac_validate::ValidateBlocking;
//!     Item::new("an-id").validate_blocking().unwrap();
//! }
//! ```
//!
//! All fetched schemas are cached, so if you're you're doing multiple
//! validations, you should re-use the same [Validator]:
//!
//! ```
//! # use stac::Item;
//! # use stac_validate::Validator;
//! let mut items: Vec<_> = (0..10).map(|n| Item::new(format!("item-{}", n))).collect();
//! # tokio_test::block_on(async {
//! let mut validator = Validator::new().await;
//! for item in items {
//!     validator.validate(&item).await.unwrap();
//! }
//! # })
//! ```
//!
//! [Validator] is cheap to clone, so you are encouraged to validate a large
//! number of objects at the same time if that's your use-case.

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
    unused_results
)]

#[cfg(feature = "blocking")]
mod blocking;
mod error;
mod validate;
mod validator;

#[cfg(feature = "blocking")]
pub use blocking::ValidateBlocking;
pub use {error::Error, validate::Validate, validator::Validator};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
use {geojson as _, rstest as _, tokio_test as _};

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
