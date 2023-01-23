//! Rust implementation of the STAC API specification.
//!
//! This crate **is**:
//!
//! - Data structures
//! - Link building
//!
//! This crate **is not**:
//!
//! - A server implementation
//!
//! For a STAC API server written in Rust, based on this crate, see
//! [stac-server-rs](http://github.com/gadomski/stac-server-rs).
//!
//! # Data structures
//!
//! Each API endpoint has its own data structure. In some cases, these are
//! light wrappers around [stac] data structures. In other cases, they can be
//! different -- e.g. the `/search` endpoint may not return [Items](stac::Item)
//! if the [fields](https://github.com/stac-api-extensions/fields) extension is
//! used, so the return type is a crate-specific [Item] struct.
//!
//! For example, here's the root structure (a.k.a the landing page):
//!
//! ```
//! use stac::Catalog;
//! use stac_api::Root;
//! let root = Root {
//!     catalog: Catalog::new("an-id", "a description"),
//!     conforms_to: vec!["https://api.stacspec.org/v1.0.0-rc.2/core".to_string()],
//! };
//! ```
//!
//! # Build links
//!
//! The [LinkBuilder] structure can build links to parts of a STAC API.
//! A [LinkBuilder] is created from a root href:
//!
//! ```
//! use stac_api::LinkBuilder;
//! let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
//! ```
//!
//! Link builders provide a variety of methods for building links to all parts of a STAC API:
//!
//! ```
//! # use stac_api::LinkBuilder;
//! # let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
//! let link = link_builder.collection_to_items("a-collection-id").unwrap();
//! assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections/a-collection-id/items");
//! ```

#![deny(missing_docs, unused_extern_crates)]

mod builder;
mod collections;
mod error;
mod fields;
mod item_collection;
mod link;
mod root;
mod search;
mod sort;

pub use {
    builder::{LinkBuilder, UrlBuilder},
    collections::Collections,
    error::Error,
    fields::Fields,
    item_collection::{Context, ItemCollection},
    link::Link,
    root::Root,
    search::Search,
    sort::Sortby,
};

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

/// A crate-specific STAC Item struct.
///
/// By default, STAC API endpoints that return [stac::Item] objects return every
/// field of those Items. However, Item objects can have hundreds of fields, or
/// large geometries, and even smaller Item objects can add up when large
/// numbers of them are in results. Frequently, not all fields in an Item are
/// used, so this specification provides a mechanism for clients to request that
/// servers to explicitly include or exclude certain fields.
#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct Item(pub serde_json::Map<String, serde_json::Value>);

impl TryFrom<stac::Item> for Item {
    type Error = serde_json::Error;

    fn try_from(item: stac::Item) -> std::result::Result<Item, serde_json::Error> {
        match serde_json::to_value(item)? {
            serde_json::Value::Object(object) => Ok(Item(object)),
            _ => panic!("a STAC item shouldn't be able to deserialize to anything but an object"),
        }
    }
}

impl TryFrom<Item> for stac::Item {
    type Error = serde_json::Error;

    fn try_from(item: Item) -> std::result::Result<stac::Item, serde_json::Error> {
        serde_json::from_value(serde_json::Value::Object(item.0))
    }
}
