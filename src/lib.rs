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
//! ```
//!
//! Attributes of STAC objects are accessed via getter and setter methods.
//! Since many attributes are shared between object types, their getters and setters are grouped into a `Core` trait that must be imported before use:
//!
//! ```
//! use stac::{Item, Core};
//! let mut item = Item::new("id");
//! assert_eq!(item.id(), "id");
//! item.set_id("new-id");
//! assert_eq!(item.id(), "new-id");
//! ```
//!
//! # Reading and writing
//!
//! Because STAC is often used for applications that require accessing remote data, this crate provides flexibility for downstream users to customize how they read and write data.
//! The [Read] trait provides an interface to turn hrefs into STAC objects.
//! The crate comes with a default [Reader] that uses the standard library for filesystem access and (if enabled) [reqwest](https://docs.rs/reqwest/latest/reqwest/) for network access:
//!
//! Because the type of STAC objects at an href cannot be known before reading, the [Read] trait returns an [Object], which is an enum wrapper around all three STAC object types.
//! [Object] implements [Core], so you can access the common STAC attributes directly from the object.
//!
//! ```
//! use stac::{Reader, Core, Read};
//! let reader = Reader::default();
//! let object = reader.read("data/catalog.json", None).unwrap();
//! assert_eq!(object.id(), "examples")
//! ```
//!
//! The crate provides a top-level [read] method for convenience:
//!
//! ```
//! let catalog = stac::read("data/catalog.json").unwrap();
//! ```
//!
//! # Tree traversal
//!
//! STAC resources are trees, where `Catalog`s and `Collection`s can contain other `Catalog`s and `Collection`s via `child` links and `Item`s via `item` links.
//! STAC objects may (but don't have to) have pointers back to their parents and the tree root, also via links.
//!
//! Tree structures in Rust can be a little tricky to implement, because Rust's strict ownership and mutability rules make storing multiple references to one object tricky.
//! This is a good thing, as tree structures in other languages (e.g. Python in PySTAC) can be hard to manage.
//! **stac-rs** provides an arena-based tree structure, inspired by [indextree](https://docs.rs/indextree/latest/indextree/), called `Stac`.
//! The `Stac` arena uses handles to point to objects in the tree, making the ergonomics slighly clumsier than a direct access tree (e.g. one based on RefCells, as described [here](https://www.nikbrendler.com/posts/rust-leetcode-primer-trees/)).
//! However, the arena tree doesn't require us to do any "interior mutability" workarounds, making this implementation hopefully easier to audit and keep correct.
//!
//! `Stac`s can be created from an href.
//! The `Stac::read` method returns both the `Stac` arena, and a handle to the read object:
//!
//! ```
//! use stac::Stac;
//! let (stac, handle) = Stac::read("data/catalog.json").unwrap();
//! ```
//!
//! This handle can be used to fetch references (mutable or immutable) to that object:
//!
//! ```
//! use stac::{Stac, Core};
//! let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
//! let catalog = stac.get(handle).unwrap();
//! assert_eq!(catalog.id(), "examples");
//! let catalog = stac.get_mut(handle).unwrap();
//! catalog.set_id("new-id");
//! let catalog = stac.get(handle).unwrap();
//! assert_eq!(catalog.id(), "new-id");
//! ```
//!
//! When objects are read into a `Stac`, their children are inserted into the tree as "unresolved" nodes.
//! They are only fetched if asked for, e.g. via the `find_child` method on the handle.
//! Note that the `Stac` object must be mutable to find children, becuase we are changing the tree by "resolving" those nodes:
//!
//! ```
//! # use stac::{Stac, Core};
//! let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
//! let child = handle
//!     .find_child(&mut stac, |child| child.id() == "sentinel-2")
//!     .unwrap()
//!     .unwrap();
//! ```
//!
//! For a more complete picture of the `Stac` object, see the [module-level documentation](stac).
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
mod core;
mod error;
mod extent;
mod item;
mod link;
mod object;
mod properties;
mod provider;
mod reader;
pub mod stac;
pub mod utils;

pub use {
    crate::core::Core,
    crate::stac::Stac,
    asset::Asset,
    catalog::{Catalog, CATALOG_TYPE},
    collection::{Collection, COLLECTION_TYPE},
    error::Error,
    extent::{Extent, SpatialExtent, TemporalExtent},
    item::{Item, ITEM_TYPE},
    link::Link,
    object::Object,
    properties::Properties,
    provider::Provider,
    reader::{Read, Reader},
};

/// The default STAC version supported by this library.
pub const STAC_VERSION: &str = "1.0.0";

/// Reads a STAC object from an HREF.
///
/// # Examples
///
/// ```
/// let catalog = stac::read("data/catalog.json").unwrap();
/// ```
pub fn read(href: &str) -> Result<Object, Error> {
    let reader = Reader::default();
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
