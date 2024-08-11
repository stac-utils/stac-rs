//! Validate STAC objects with jsonschema.
//!
//! # Examples
//!
//! Validation is provided via the [Validate] trait:
//!
//! ```
//! use stac::Item;
//! use stac_validate::Validate;
//! Item::new("an-id").validate().unwrap();
//! ```
//!
//! [stac::Collection], [stac::Catalog], and [stac::Item] all have their schemas built into the library, so they don't need to be fetched from the network.
//! Any extension schemas are fetched using [reqwest](https://docs.rs/reqwest/latest/reqwest/), and cached for later use.
//! This means that, if you're doing multiple validations, you should re-use the same [Validator]:
//!
//! ```
//! # use stac::Item;
//! use stac_validate::Validator;
//!
//! let mut items: Vec<_> = (0..10).map(|n| Item::new(format!("item-{}", n))).collect();
//! let mut validator = Validator::new();
//! for item in items {
//!     validator.validate(item).unwrap();
//! }
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

mod error;
mod validate;
mod validator;

pub use {
    error::Error,
    validate::{Validate, ValidateCore},
    validator::Validator,
};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

#[cfg(test)]
mod tests {
    use crate::Validate;
    use geojson::{Geometry, Value};
    use rstest as _;
    use stac::{Catalog, Collection, Item};

    #[test]
    fn item() {
        let item = Item::new("an-id");
        item.validate().unwrap();
    }

    #[test]
    fn item_with_geometry() {
        let mut item = Item::new("an-id");
        item.set_geometry(Geometry::new(Value::Point(vec![-105.1, 40.1])))
            .unwrap();
        item.validate().unwrap();
    }

    #[test]
    fn item_with_extensions() {
        let item: Item =
            stac::read("data/extensions-collection/proj-example/proj-example.json").unwrap();
        item.validate().unwrap();
    }

    #[test]
    fn catalog() {
        let catalog = Catalog::new("an-id", "a description");
        catalog.validate().unwrap();
    }

    #[test]
    fn collection() {
        let collection = Collection::new("an-id", "a description");
        collection.validate().unwrap();
    }

    #[test]
    fn value() {
        let value: stac::Value = stac::read("data/simple-item.json").unwrap();
        value.validate().unwrap();
    }

    #[test]
    fn item_collection() {
        let item = stac::read("data/simple-item.json").unwrap();
        let item_collection = stac::ItemCollection::from(vec![item]);
        item_collection.validate().unwrap();
    }
}

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
