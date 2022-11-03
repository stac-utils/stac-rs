use crate::{
    link::COLLECTION_REL, media_type::JSON, Asset, Collection, Error, Href, Link, Properties,
    Result, STAC_VERSION,
};
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// The type field for [Items](Item).
pub const ITEM_TYPE: &str = "Feature";

/// An `Item` is a GeoJSON Feature augmented with foreign members relevant to a
/// STAC object.
///
/// These include fields that identify the time range and assets of the `Item`. An
/// `Item` is the core object in a STAC catalog, containing the core metadata that
/// enables any client to search or crawl online catalogs of spatial 'assets'
/// (e.g., satellite imagery, derived data, DEMs).
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Item {
    /// Type of the GeoJSON Object. MUST be set to `"Feature"`.
    pub r#type: String,

    /// The STAC version the `Item` implements.
    #[serde(rename = "stac_version")]
    pub version: String,

    /// A list of extensions the `Item` implements.
    #[serde(rename = "stac_extensions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Provider identifier.
    ///
    /// The ID should be unique within the [Collection](crate::Collection) that contains the `Item`.
    pub id: String,

    /// Defines the full footprint of the asset represented by this item,
    /// formatted according to [RFC 7946, section
    /// 3.1](https://tools.ietf.org/html/rfc7946#section-3.1).
    ///
    /// The footprint should be the default GeoJSON geometry, though additional
    /// geometries can be included. Coordinates are specified in
    /// Longitude/Latitude or Longitude/Latitude/Elevation based on [WGS
    /// 84](http://www.opengis.net/def/crs/OGC/1.3/CRS84).
    pub geometry: Option<Geometry>,

    /// Bounding Box of the asset represented by this `Item`, formatted according
    /// to [RFC 7946, section 5](https://tools.ietf.org/html/rfc7946#section-5).
    ///
    /// REQUIRED if `geometry` is not `null`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<f64>>,

    /// A dictionary of additional metadata for the `Item`.
    pub properties: Properties,

    /// List of link objects to resources and related URLs.
    pub links: Vec<Link>,

    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    pub assets: HashMap<String, Asset>,

    /// The `id` of the STAC [Collection](crate::Collection) this `Item`
    /// references to.
    ///
    /// This field is *required* if such a relation type is present and is *not
    /// allowed* otherwise. This field provides an easy way for a user to search
    /// for any `Item`s that belong in a specified `Collection`. Must be a non-empty
    /// string.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,

    /// Additional fields not part of the Item specification.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,

    #[serde(skip)]
    href: Option<String>,
}

impl Item {
    /// Creates a new `Item` with the given `id`.
    ///
    /// The item properties' `datetime` field is set to the object creation
    /// time.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// let item = Item::new("an-id");
    /// assert_eq!(item.id, "an-id");
    /// ```
    pub fn new(id: impl ToString) -> Item {
        Item {
            r#type: ITEM_TYPE.to_string(),
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

    /// Sets this item's [Collection], both setting the `collection` field and adding a link with a `collection` rel type.
    ///
    /// If the Collection does not have an href, returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Href, Collection};
    /// let mut item = Item::new("an-item");
    /// let mut collection = Collection::new("a-collection");
    /// collection.set_href("a/href");
    /// item.set_collection(&collection).unwrap();
    /// ```
    pub fn set_collection(&mut self, collection: &Collection) -> Result<()> {
        if let Some(href) = collection.href() {
            self.collection = Some(collection.id.clone());
            self.links.retain(|link| !link.is_collection());
            self.links.push(Link {
                href: href.to_string(),
                rel: COLLECTION_REL.to_string(),
                r#type: Some(JSON.to_string()),
                title: collection.title.clone(),
                additional_fields: Default::default(),
            });
            Ok(())
        } else {
            Err(Error::MissingHref(collection.id.clone()))
        }
    }

    /// Returns this item's collection link.
    ///
    /// If there are zero or more than one link with rel "collection", returns [None].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Link};
    /// let mut item = Item::new("an-id");
    /// assert!(item.collection_link().is_none());
    /// item.links.push(Link::new("a/href", "collection"));
    /// assert!(item.collection_link().is_some());
    /// item.links.push(Link::new("a/second/href", "collection"));
    /// assert!(item.collection_link().is_none());
    /// ```
    pub fn collection_link(&self) -> Option<&Link> {
        let mut iter = self.links.iter().filter(|link| link.is_collection());
        if let Some(link) = iter.next() {
            if iter.next().is_none() {
                Some(link)
            } else {
                None
            }
        } else {
            None
        }
    }
}

impl Href for Item {
    fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    fn set_href(&mut self, href: impl ToString) {
        self.href = Some(href.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::Item;
    use crate::{media_type::JSON, Collection, Error, Href, Link, STAC_VERSION};

    #[test]
    fn new() {
        let item = Item::new("an-id");
        assert_eq!(item.geometry, None);
        assert!(item.properties.datetime.is_some());
        assert!(item.assets.is_empty());
        assert!(item.collection.is_none());
        assert_eq!(item.r#type, "Feature");
        assert_eq!(item.version, STAC_VERSION);
        assert!(item.extensions.is_none());
        assert_eq!(item.id, "an-id");
        assert!(item.links.is_empty());
    }

    #[test]
    fn skip_serializing() {
        let item = Item::new("an-id");
        let value = serde_json::to_value(item).unwrap();
        assert!(value.get("stac_extensions").is_none());
        assert!(value.get("bbox").is_none());
        assert!(value.get("collection").is_none());
    }

    #[test]
    fn set_collection_without_href() {
        let mut item = Item::new("an-id");
        let collection = Collection::new("a-collection");
        assert!(matches!(
            item.set_collection(&collection).unwrap_err(),
            Error::MissingHref(_)
        ))
    }

    #[test]
    fn set_collection() {
        let mut item = Item::new("an-id");
        let mut collection = Collection::new("a-collection");
        collection.set_href("a/href");
        item.set_collection(&collection).unwrap();
        assert_eq!(item.collection.as_deref().unwrap(), "a-collection");
        let link = item.collection_link().unwrap();
        assert_eq!(link.href, "a/href");
        assert_eq!(link.r#type.as_deref().unwrap(), JSON);
    }

    #[test]
    fn set_collection_clears_old_link() {
        let mut item = Item::new("an-id");
        let mut collection = Collection::new("a-collection");
        collection.set_href("a/href");
        item.set_collection(&collection).unwrap();
        collection.id = "a-second-id".to_string();
        collection.set_href("a/second/href");
        item.set_collection(&collection).unwrap();
        assert_eq!(item.collection.as_deref().unwrap(), "a-second-id");
        let link = item.collection_link().unwrap();
        assert_eq!(link.href, "a/second/href");
    }

    #[test]
    fn collection_link_no_links() {
        let item = Item::new("an-id");
        assert!(item.collection_link().is_none());
    }

    #[test]
    fn collection_link_one() {
        let mut item = Item::new("an-id");
        item.links.push(Link::new("a/href", "collection"));
        let link = item.collection_link().unwrap();
        assert_eq!(link.href, "a/href")
    }

    #[test]
    fn collection_link_two() {
        let mut item = Item::new("an-id");
        item.links.push(Link::new("a/href", "collection"));
        item.links.push(Link::new("a/second/href", "collection"));
        assert!(item.collection_link().is_none());
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
