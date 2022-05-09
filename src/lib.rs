//! Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.
//!
//! The SpatioTemporal Asset Catalog (STAC) specification provides a common language to describe a range of geospatial information, so it can more easily be indexed and discovered.
//! A 'spatiotemporal asset' is any file that represents information about the earth captured in a certain space and time.
//!
//! This is a Rust implementation of the specification, with associated utilities.
//! Similar projects in other languages include:
//!
//! - Python: [PySTAC](https://pystac.readthedocs.io/en/1.0/)
//! - Go: [go-stac](https://github.com/planetlabs/go-stac)
//! - .NET: [DotNetStac](https://github.com/Terradue/DotNetStac)
//! - PHP: [resto](https://github.com/jjrom/resto)
//!
//! # Basic data structures
//!
//! STAC has three data structures:
//!
//! - [Item] is a [GeoJSON](http://geojson.org/) [Feature](https://tools.ietf.org/html/rfc7946#section-3.2) augmented with [foreign members](https://tools.ietf.org/html/rfc7946#section-6)
//! - [Catalog] represents a logical group of other `Catalogs`, `Collections`, and `Items`
//! - [Collection] shares all fields with the `Catalog` (with different allowed values for `type` and `stac_extensions`) and adds fields to describe the whole dataset and the included set of `Items`
//!
//! All three are provided as [serde](https://serde.rs/) (de)serializable structures with public attributes.
//! Each structure provides a `new` method that takes an `id` and fills the rest of the object's attributes with sensible defaults:
//!
//! ```
//! use stac::{Item, Catalog, Collection};
//! let item = Item::new("id");
//! let catalog = Catalog::new("id");
//! let collection = Catalog::new("id");
//! ```
//!
//! # Reading and writing
//!
//! [Reader] uses the standard library for filesystem access:
//!
//! ```
//! use stac::{Reader, Read};
//! let catalog = Reader::default().read("data/catalog.json").unwrap();
//! ```
//!
//! If the [reqwest](https://docs.rs/reqwest/latest/reqwest/) feature is enabled, it is used for network access:
//!
//! ```no_run
//! # use stac::{Reader, Read};
//! let catalog = Reader::default().read("http://example.com/stac/catalog.json").unwrap();
//! ```
//!
//! Because the type of a STAC object cannot be known before reading, reading returns an [HrefObject], which is an [Href] and an [Object]:
//!
//! ```
//! # use stac::{Reader, Read};
//! let reader = Reader::default();
//! let href_object = reader.read("data/catalog.json").unwrap();
//!
//! let object = href_object.object;
//! assert_eq!(object.id(), "examples");
//! let catalog = object.as_catalog().unwrap();
//! assert_eq!(catalog.title.as_ref().unwrap(), "Example Catalog");
//!
//! let href = href_object.href;
//! assert_eq!(href.as_str(), "data/catalog.json");
//! ```
//!
//! There is a top-level [read()] method for convenience:
//!
//! ```
//! let catalog = stac::read("data/catalog.json").unwrap();
//! ```
//!
//! [Writer] only knows how to write to the local filesystem -- writing to a url is an error:
//!
//! ```no_run
//! use stac::{Item, HrefObject, Writer, Write};
//! let item = Item::new("an-id");
//! let object = HrefObject::new(item, "item.json");
//! let writer = Writer::default();
//! writer.write(object).unwrap();
//!
//! let item = Item::new("an-id");
//! let object = HrefObject::new(item, "http://example.com/item.json");
//! writer.write(object).unwrap_err();
//! ```
//!
//! If you need more functionality than what is provided by [Reader] and [Writer], you can implement the [Read] or [Write] traits.
//!
//! # STAC catalogs
//!
//! Throughout the STAC spec, `catalog` (with a lower-case `c`) is used to refer to entire trees of STAC Catalogs, Collections, and Items.
//! STAC catalogs (with a lower-case `c`) are supported via the [Stac] structure.
//! See the [stac] module documentation for more information on how to read, create, modify, and write STAC catalogs.
//!
//! ```no_run
//! use stac::{Layout, Stac, Catalog, Item, Writer};
//! let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
//! let _ = stac.add_child(root, Item::new("child-item"));
//! let mut layout = Layout::new("the/root/directory");
//! let writer = Writer::default();
//! // Writes the stac to
//! // - `the/root/directory/catalog.json`
//! // - `the/root/directory/child-item/child-item.json`
//! // with the appropriate links between the objects.
//! stac.write(&mut layout, &writer).unwrap();
//! ```
//!
//! # Other features
//!
//! - The [Href] enum provides a wrapper around remote and local hrefs and paths to ensure cross-platform compatibility.
//! - The source repository contains canonical examples copied the [stac-spec repository](https://github.com/radiantearth/stac-spec/tree/master/examples), and these examples are tested for round trip equality.
//!   For example:
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
    unused_results
)]

mod asset;
mod catalog;
mod collection;
mod error;
mod extent;
mod href;
mod item;
pub mod layout;
mod link;
pub mod media_type;
mod object;
mod properties;
mod provider;
mod read;
pub mod stac;
mod write;

pub use {
    crate::stac::{Handle, Stac, Walk},
    asset::Asset,
    catalog::{Catalog, CATALOG_TYPE},
    collection::{Collection, COLLECTION_TYPE},
    error::Error,
    extent::{Extent, SpatialExtent, TemporalExtent},
    href::Href,
    item::{Item, ITEM_TYPE},
    layout::Layout,
    link::Link,
    object::{HrefObject, Object, ObjectHrefTuple},
    properties::Properties,
    provider::Provider,
    read::{Read, Reader},
    write::{Write, Writer},
};

/// The default STAC version supported by this library.
pub const STAC_VERSION: &str = "1.0.0";

/// Custom [Result](std::result::Result) type for this crate.
pub type Result<T> = std::result::Result<T, Error>;

/// Reads a STAC object from an href.
///
/// # Examples
///
/// ```
/// let catalog = stac::read("data/catalog.json").unwrap();
/// ```
pub fn read(href: impl Into<Href>) -> Result<HrefObject> {
    let reader = Reader::default();
    reader.read(href)
}

/// Reads a [Catalog] from an [Href].
///
/// # Examples
///
/// ```
/// use stac::Href;
/// let catalog = stac::read_catalog(&Href::new("data/catalog.json")).unwrap();
/// ```
pub fn read_catalog(href: &Href) -> Result<Catalog> {
    let reader = Reader::default();
    reader.read_object(href)
}

/// Reads a [Collection] from an [Href].
///
/// # Examples
///
/// ```
/// use stac::Href;
/// let collection = stac::read_collection(&Href::new("data/collection.json")).unwrap();
/// ```
pub fn read_collection(href: &Href) -> Result<Collection> {
    let reader = Reader::default();
    reader.read_object(href)
}

/// Reads an [Item] from an [Href].
///
/// # Examples
///
/// ```
/// use stac::Href;
/// let item = stac::read_item(&Href::new("data/simple-item.json")).unwrap();
/// ```
pub fn read_item(href: &Href) -> Result<Item> {
    let reader = Reader::default();
    reader.read_object(href)
}

#[cfg(test)]
mod tests {
    use criterion as _;

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
