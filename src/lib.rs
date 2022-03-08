//! Read and write [SpatioTemporal Asset Catalogs (STACs)](https://stacspec.org/) in Rust.
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
//! # Basic data strutures
//!
//! STAC is built on three data structures:
//!
//! - [Item] is a [GeoJSON](http://geojson.org/) [Feature](https://tools.ietf.org/html/rfc7946#section-3.2) augmented with [foreign members](https://tools.ietf.org/html/rfc7946#section-6) relevant to a STAC object.
//! - [Catalog] represents a logical group of other `Catalogs`, `Collections`, and `Items`.
//! - [Collection] shares all fields with the `Catalog` (with different allowed values for `type` and `stac_extensions`) and adds fields to describe the whole dataset and the included set of `Items`.
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
//! The provided [Reader] uses the standard library for filesystem access:
//!
//! ```
//! use stac::{Reader, Read};
//! let catalog = Reader::default().read("data/catalog.json").unwrap();
//! ```
//!
//! If the [reqwest](https://docs.rs/reqwest/latest/reqwest/) feature is enabled (it is enabled by default), it is used for network access:
//!
//! ```no_run
//! # use stac::{Reader, Read};
//! let catalog = Reader::default().read("http://example.com/stac/catalog.json").unwrap();
//! ```
//!
//! Because the type of a STAC object cannot be known before reading, a read returns an [HrefObject], which is a [Href] and an [Object]:
//!
//! ```
//! # use stac::{Reader, Read};
//! let reader = Reader::default();
//! let read_object = reader.read("data/catalog.json").unwrap();
//!
//! let object = read_object.object;
//! assert_eq!(object.id(), "examples");
//! let catalog = object.as_catalog().unwrap();
//! assert_eq!(catalog.title.as_ref().unwrap(), "Example Catalog");
//!
//! let href = read_object.href;
//! assert_eq!(href.as_str(), "data/catalog.json");
//! ```
//!
//! There is a top-level [read()] method for convenience:
//!
//! ```
//! let catalog = stac::read("data/catalog.json").unwrap();
//! ```
//!
//! The built-in [Writer] only knows how to write to the local filesystem -- writing to a url is an error:
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
//! If you need more functionality than is provided by [Reader] and [Writer], your own structures can implement the [Read] or [Write] traits themselves.
//!
//! # STAC catalogs are trees
//!
//! Because of Rust's strict mutability and ownership rules, tree structures require more verbose ergonomics than in other languages.
//! Our [Stac] is an arena-based tree inspired by [indextree](https://docs.rs/indextree/latest/indextree/).
//! The `Stac` arena uses handles to point to objects in the tree.
//!
//! A `Stac` can be created from an href or an object.
//! When you create a `Stac`, you get back the `Stac` and a [Handle] to that object:
//!
//! ```
//! use stac::{Stac, Catalog};
//! let (stac, handle) = Stac::read("data/catalog.json").unwrap();
//! let (stac, handle) = Stac::new(Catalog::new("root")).unwrap();
//! ```
//!
//! `Stac` is a lazy cache, meaning that it doesn't read objects until needed, and keeps read objects in a cache keyed by their hrefs.
//! Objects are read on-demand, e.g. via the [get](Stac::get) method, and any future access returns the stored object, instead of reading it again:
//!
//! ```
//! # use stac::Stac;
//! let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
//! let children = stac.children(root); // <- none have the children have been read yet
//! let child = stac.get(children[0]).unwrap(); // <- the first child is now read into the `Stac`
//! let child = stac.get(children[0]).unwrap(); // <- does not do any additional reads
//! ```
//!
//! ## Layout
//!
//! The structure of a STAC catalog is defined by its [Links](Link).
//! The process of translating a [Stac] tree into a set of `child`, `item`, `parent`, and `root` links is handled by [Layout].
//! By default, a `Layout` uses the [best practices](https://github.com/radiantearth/stac-spec/blob/master/best-practices.md#catalog-layout) provided by the STAC specification:
//!
//! ```
//! use stac::{Stac, Layout, Catalog, Collection, Item};
//! let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
//! let collection = stac.add_child(root, Collection::new("the-collection")).unwrap();
//! let item = stac.add_child(collection, Item::new("an-item")).unwrap();
//! let mut layout = Layout::new("my/stac/v0");
//! layout.layout(&mut stac).unwrap(); // <- sets each object's href and creates links
//! assert_eq!(
//!     stac.href(root).unwrap().as_str(),
//!     "my/stac/v0/catalog.json"
//! );
//! assert_eq!(
//!     stac.href(collection).unwrap().as_str(),
//!     "my/stac/v0/the-collection/collection.json"
//! );
//! assert_eq!(
//!     stac.href(item).unwrap().as_str(),
//!     "my/stac/v0/the-collection/an-item/an-item.json"
//! );
//! ```
//!
//! ## Rendering and writing
//!
//! To avoid unnecessary copying, the [Layout::render] method moves the [Hrefs](Href) and [Objects](Object) out of a [Stac], e.g. for writing.
//! This can be done via an iterator, which means you can read, layout, and write an entire STAC catalog without ever having to load it all into memory:
//!
//! ```no_run
//! use stac::{Stac, Layout, Writer, Write};
//! let (stac, _) = Stac::read("data/catalog.json").unwrap();
//! let mut layout = Layout::new("my/stac/v0");
//! let writer = Writer::default();
//! for result in layout.render(stac) {
//!     let href_object = result.unwrap();
//!     writer.write(href_object).unwrap();
//! }
//! ```
//!
//! [Stac::write] is a convenience method that works just like this.
//!
//! # Roundtrip equality
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
    href::{Href, PathBufHref},
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
pub fn read(href: impl Into<PathBufHref>) -> Result<HrefObject> {
    let reader = Reader::default();
    reader.read(href)
}

/// Reads a [Catalog] from an [Href].
///
/// # Examples
///
/// ```
/// let catalog = stac::read_catalog("data/catalog.json").unwrap();
/// ```
pub fn read_catalog(href: impl Into<PathBufHref>) -> Result<Catalog> {
    let reader = Reader::default();
    reader.read_struct(href.into())
}

/// Reads a [Collection] from an [Href].
///
/// # Examples
///
/// ```
/// let collection = stac::read_collection("data/collection.json").unwrap();
/// ```
pub fn read_collection(href: impl Into<PathBufHref>) -> Result<Collection> {
    let reader = Reader::default();
    reader.read_struct(href.into())
}

/// Reads an [Item] from an [Href].
///
/// # Examples
///
/// ```
/// let item = stac::read_item("data/simple-item.json").unwrap();
/// ```
pub fn read_item(href: impl Into<PathBufHref>) -> Result<Item> {
    let reader = Reader::default();
    reader.read_struct(href.into())
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
