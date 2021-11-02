//! Reads and writes SpatioTemporal Asset Catalogs (STACs) in Rust.
//!
//! # Basic data strutures
//!
//! STAC is built on three data structures:
//!
//! - [Item](https://github.com/radiantearth/stac-spec/blob/master/item-spec/item-spec.md)
//! - [Catalog](https://github.com/radiantearth/stac-spec/blob/master/catalog-spec/catalog-spec.md)
//! - [Collection](https://github.com/radiantearth/stac-spec/blob/master/collection-spec/collection-spec.md)
//!
//! All three structures are provided as [serde](https://serde.rs/) (de)serializable structures with public attributes.
//! Because `id` is always required, the structures do not implement `Default`.
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
//! let file = File::open("examples/simple-item.json").unwrap();
//! let buf_reader = BufReader::new(file);
//! let before: Value = serde_json::from_reader(buf_reader).unwrap();
//! let item: Item = serde_json::from_value(before.clone()).unwrap();
//! let after = serde_json::to_value(item).unwrap();
//! assert_eq!(before, after);
//! ```

mod asset;
mod catalog;
mod collection;
mod extent;
mod item;
mod link;
mod provider;

pub use {
    asset::Asset, catalog::Catalog, collection::Collection, extent::Extent, item::Item, link::Link,
    provider::Provider,
};

pub const STAC_VERSION: &str = "1.0.0";

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
