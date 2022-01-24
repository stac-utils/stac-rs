use crate::{
    core::{Core, CoreStruct},
    Asset, Properties,
};
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// The type field for Items.
pub const ITEM_TYPE: &str = "Feature";

/// An Item is a GeoJSON Feature augmented with foreign members relevant to a
/// STAC object.
///
/// These include fields that identify the time range and assets of the Item. An
/// Item is the core object in a STAC Catalog, containing the core metadata that
/// enables any client to search or crawl online catalogs of spatial 'assets'
/// (e.g., satellite imagery, derived data, DEMs).
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Item {
    #[serde(flatten)]
    core: CoreStruct,

    /// Defines the full footprint of the asset represented by this item, formatted according to [RFC 7946, section 3.1](https://tools.ietf.org/html/rfc7946#section-3.1).
    ///
    /// The footprint should be the default GeoJSON geometry, though additional geometries can be included.
    /// Coordinates are specified in Longitude/Latitude or Longitude/Latitude/Elevation Cored on [WGS 84](http://www.opengis.net/def/crs/OGC/1.3/CRS84).
    pub geometry: Option<Geometry>,

    /// Bounding Box of the asset represented by this Item, formatted according
    /// to RFC 7946, section 5.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<f64>>,

    /// A dictionary of additional metadata for the Object.
    pub properties: Properties,

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
}

impl Item {
    /// Creates a new `Item` with the given `id`.
    ///
    /// The item properties' `datetime` field is set to the creation time.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Core};
    /// let item = Item::new("an-id");
    /// assert_eq!(item.id(), "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Item {
        Item {
            core: CoreStruct::new(ITEM_TYPE, id),
            geometry: None,
            bbox: None,
            properties: Properties::default(),
            assets: HashMap::new(),
            collection: None,
        }
    }

    /// Returns a reference to this Item's geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Item;
    /// let item = Item::new("an-id");
    /// assert!(item.geometry().is_none());
    /// ```
    pub fn geometry(&self) -> Option<&Geometry> {
        self.geometry.as_ref()
    }

    /// Returns a reference to this Item's properties.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Item, Properties};
    /// let item = Item::new("an-id");
    /// assert!(item.properties().datetime.is_some());
    /// ```
    pub fn properties(&self) -> &Properties {
        &self.properties
    }

    /// Returns a reference to this Item's assets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Item;
    /// let item = Item::new("an-id");
    /// assert!(item.assets().is_empty());
    /// ```
    pub fn assets(&self) -> &HashMap<String, Asset> {
        &self.assets
    }

    /// Returns a reference to this Item's collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Item;
    /// let item = Item::new("an-id");
    /// assert!(item.collection().is_none());
    /// ```
    pub fn collection(&self) -> Option<&str> {
        self.collection.as_deref()
    }
}

impl AsRef<CoreStruct> for Item {
    fn as_ref(&self) -> &CoreStruct {
        &self.core
    }
}

impl AsMut<CoreStruct> for Item {
    fn as_mut(&mut self) -> &mut CoreStruct {
        &mut self.core
    }
}

impl Core for Item {}

#[cfg(test)]
mod tests {
    use super::Item;
    use crate::{Core, STAC_VERSION};

    #[test]
    fn new() {
        let item = Item::new("an-id");
        assert_eq!(item.geometry(), None);
        assert!(item.properties().datetime.is_some());
        assert!(item.assets().is_empty());
        assert!(item.collection().is_none());
        assert_eq!(item.type_(), "Feature");
        assert_eq!(item.version(), STAC_VERSION);
        assert!(item.extensions().is_none());
        assert_eq!(item.id(), "an-id");
        assert!(item.links().is_empty());
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
