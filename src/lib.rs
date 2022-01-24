//! Reads and writes [SpatioTemporal Asset Catalogs (STACs)](https://stacspec.org/) in Rust.
//!
//! The SpatioTemporal Asset Catalog (STAC) specification provides a common language to describe a range of geospatial information, so it can more easily be indexed and discovered.
//! A 'spatiotemporal asset' is any file that represents information about the earth captured in a certain space and time.
//!
//! The goal is for all providers of spatiotemporal assets (Imagery, SAR, Point Clouds, Data Cubes, Full Motion Video, etc) to expose their data as SpatioTemporal Asset Catalogs (STAC), so that new code doesn't need to be written whenever a new data set or API is released.
//!
//! This is a Rust implementation of the specification, with associated utilties.
//! Similar projects in other languages include:
//!
//! - Python: [PySTAC](https://pystac.readthedocs.io/en/1.0/)
//! - Go: [go-stac](https://github.com/planetlabs/go-stac)
//! - .NET: [DotNetStac](https://github.com/Terradue/DotNetStac)
//! - PHP: [resto](https://github.com/jjrom/resto)
//!
//! # Basic data strutures
//!
//! STAC is built on three data structures:
//!
//! - [Item](https://github.com/radiantearth/stac-spec/blob/master/item-spec/item-spec.md) is a [GeoJSON](http://geojson.org/) [Feature](https://tools.ietf.org/html/rfc7946#section-3.2) augmented with [foreign members](https://tools.ietf.org/html/rfc7946#section-6) relevant to a STAC object.
//! - [Catalog](https://github.com/radiantearth/stac-spec/blob/master/catalog-spec/catalog-spec.md) represents a logical group of other Catalog, Collection, and Item objects.
//! - [Collection](https://github.com/radiantearth/stac-spec/blob/master/collection-spec/collection-spec.md) shares all fields with the Catalog (with different allowed values for `type` and `stac_extensions`) and adds fields to describe the whole dataset and the included set of Items.
//!
//! All three structures are provided as [serde](https://serde.rs/) (de)serializable structures with public attributes.
//! Because `id` is always required, these structures do not implement `Default`.
//! Each provides a `new` method that takes an `id` and fills the rest with sensible defaults.
//! Fields that have a `stac_*` prefix are stripped down to the suffix, e.g. `stac_version` becomes `version`.
//!
//! ```
//! use stac::{Item, Catalog, Collection};
//! let item = Item::new("id");
//! let catalog = Catalog::new("id");
//! let collection = Catalog::new("id");
//! assert_eq!(item.version, "1.0.0");
//! ```
//!
//! # Full specification compliance
//!
//! The source repository contains canonical examples copied the [stac-spec repository](https://github.com/radiantearth/stac-spec/tree/master/examples), and these examples are tested for round trip equality.
//! For example:
//!
//! ```
//! use std::fs::File;
//! use std::io::BufReader;
//! use std::str::FromStr;
//! use serde_json::Value;
//! use stac::Item;
//!
//! let file = File::open("data/simple-item.json").unwrap();
//! let buf_reader = BufReader::new(file);
//! let before: Value = serde_json::from_reader(buf_reader).unwrap();
//! let item: Item = serde_json::from_value(before.clone()).unwrap();
//! let after = serde_json::to_value(item).unwrap();
//! assert_eq!(before, after);
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
    unused_results,
    variant_size_differences
)]

mod asset;
mod catalog;
mod collection;
mod error;
mod extent;
mod item;
mod link;
mod object;
mod properties;
mod provider;
mod reader;
pub mod utils;

pub use {
    asset::Asset,
    catalog::Catalog,
    collection::Collection,
    error::Error,
    extent::{Extent, SpatialExtent, TemporalExtent},
    item::Item,
    link::Link,
    object::Object,
    properties::Properties,
    provider::Provider,
    reader::Reader,
};

/// The default STAC version supported by this library.
pub const STAC_VERSION: &str = "1.0.0";

/// Reads a STAC object from an HREF.
///
/// # Examples
///
/// ```
/// let catalog: stac::Catalog = stac::read("data/catalog.json").unwrap();
/// ```
pub fn read<O: Object>(href: &str) -> Result<O, Error> {
    let reader = Reader::new();
    reader.read(href, None)
}

#[cfg(test)]
mod tests {
    macro_rules! roundtrip {
        ($function:ident, $filename:expr, $object:ident) => {
            #[test]
            fn $function() {
                use assert_json_diff::assert_json_eq;
                use serde_json::Value;
                use std::fs::File;
                use std::io::BufReader;

                let file = File::open($filename).unwrap();
                let buf_reader = BufReader::new(file);
                let before: Value = serde_json::from_reader(buf_reader).unwrap();
                let object: $object = serde_json::from_value(before.clone()).unwrap();
                let after = serde_json::to_value(object).unwrap();
                assert_json_eq!(before, after);
            }
        };
    }
    pub(crate) use roundtrip;
}
