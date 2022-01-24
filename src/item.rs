use crate::{Asset, Link, Properties, STAC_VERSION};
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

pub(crate) const ITEM_TYPE: &str = "Feature";

/// An Item is a GeoJSON Feature augmented with foreign members relevant to a
/// STAC object.
///
/// These include fields that identify the time range and assets of the Item. An
/// Item is the core object in a STAC Catalog, containing the core metadata that
/// enables any client to search or crawl online catalogs of spatial 'assets'
/// (e.g., satellite imagery, derived data, DEMs).
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Item {
    /// Type of the GeoJSON Object. MUST be set to `Feature`.
    #[serde(rename = "type")]
    pub type_: String,

    /// The STAC version the Item implements.
    #[serde(rename = "stac_version")]
    pub version: String,

    /// A list of extensions the Item implements.
    #[serde(rename = "stac_extensions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Provider identifier.
    ///
    /// The ID should be unique within the Collection that contains the Item.
    pub id: String,

    /// Defines the full footprint of the asset represented by this item, formatted according to [RFC 7946, section 3.1](https://tools.ietf.org/html/rfc7946#section-3.1).
    ///
    /// The footprint should be the default GeoJSON geometry, though additional geometries can be included.
    /// Coordinates are specified in Longitude/Latitude or Longitude/Latitude/Elevation based on [WGS 84](http://www.opengis.net/def/crs/OGC/1.3/CRS84).
    pub geometry: Option<Geometry>,

    /// Bounding Box of the asset represented by this Item, formatted according
    /// to RFC 7946, section 5.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<f64>>,

    /// A dictionary of additional metadata for the Item.
    pub properties: Properties,

    /// List of link objects to resources and related URLs.
    ///
    /// A link with the `rel` set to `self` is strongly recommended.
    pub links: Vec<Link>,

    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    pub assets: HashMap<String, Asset>,

    /// The id of the STAC Collection this Item references to (see collection relation type).
    ///
    /// This field is required if such a relation type is present and is not
    /// allowed otherwise. This field provides an easy way for a user to search
    /// for any Items that belong in a specified Collection. Must be a non-empty
    /// string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,

    /// Additional fields on the Item.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,

    /// The href from which the Item was read.
    ///
    /// Not serialized.
    #[serde(skip)]
    pub(crate) href: Option<String>,
}

impl Item {
    /// Creates a new `Item` with the given `id`.
    ///
    /// The item properties' `datetime` field is set to the creation time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Item;
    /// let item = Item::new("an-id");
    /// assert_eq!(item.id, "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Item {
        Item {
            type_: ITEM_TYPE.to_string(),
            version: STAC_VERSION.to_string(),
            extensions: None,
            id: id.to_string(),
            geometry: None,
            bbox: None,
            properties: Properties::default(),
            links: Vec::new(),
            assets: HashMap::new(),
            collection: None,
            additional_fields: Map::new(),
            href: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Item;
    use crate::STAC_VERSION;

    #[test]
    fn new() {
        let item = Item::new("an-id");
        assert_eq!(item.type_, "Feature");
        assert_eq!(item.version, STAC_VERSION);
        assert!(item.extensions.is_none());
        assert_eq!(item.id, "an-id");
        assert_eq!(item.geometry, None);
        assert!(item.properties.datetime.is_some());
        assert!(item.links.is_empty());
        assert!(item.assets.is_empty());
        assert!(item.collection.is_none());
    }

    #[test]
    fn skip_serializing() {
        let item = Item::new("an-id");
        let value = serde_json::to_value(item).unwrap();
        assert!(value.get("stac_extensions").is_none());
        assert!(value.get("bbox").is_none());
        assert!(value.get("collection").is_none());
    }
    mod roundtrip {
        use super::Item;
        use crate::tests::roundtrip;

        roundtrip!(simple_item, "data/simple-item.json", Item);
        roundtrip!(extended_item, "data/extended-item.json", Item);
        roundtrip!(core_item, "data/core-item.json", Item);
        roundtrip!(collectionless_item, "data/collectionless-item.json", Item);
        roundtrip!(
            proj_example_item,
            "data/extensions-collection/proj-example/proj-example.json",
            Item
        );
    }
}
