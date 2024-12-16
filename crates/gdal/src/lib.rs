#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![deny(
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_debug_implementations,
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

//! Use [gdal] to instrument [stac] and [stac_extensions] crates.
//! ## Usage:
//! ```
//! use stac::{Asset, Item};
//! use stac_extensions::{Extensions, Raster};
//!
//! let mut item = Item::new("an-id");
//! item.assets.insert("data".to_string(), Asset::new("assets/dataset_geo.tif"));
//!
//! stac_gdal::update_item(&mut item, false, true).unwrap();
//!
//! assert!(item.has_extension::<Raster>());
//! ```

mod error;

/// Update [stac::Item] using GDAL functions.
pub mod item;

/// Instrument [stac_extensions::Projection] with GDAL based calculations.
pub mod projection;

/// Crate-specific [Error](std::error::Error) type.
pub use error::Error;

/// Crate-specific [Result](std::result::Result) type.
pub type Result<T> = std::result::Result<T, Error>;

pub use item::update_item;
pub use projection::ProjectionCalculations;
