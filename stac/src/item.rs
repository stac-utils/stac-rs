use crate::{Asset, Assets, Error, Extensions, Href, Link, Links, Result, STAC_VERSION};
use chrono::Utc;
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

    /// Type of the GeoJSON Object. MUST be set to `"Feature"`.
    #[serde(
        deserialize_with = "deserialize_type",
        serialize_with = "serialize_type"
    )]
    r#type: String,

    /// The STAC version the `Item` implements.
    #[serde(rename = "stac_version")]
    version: String,

    #[serde(skip)]
    href: Option<String>,
}

/// Additional metadata fields can be added to the GeoJSON Object Properties.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Geometry {
    /// The geometry type.
    pub r#type: String,

    /// The other geometry attributes.
    ///
    /// `GeometryCollection` doesn't have a `coordinates` member, so we must
    /// capture everything in a flat, generic array.
    #[serde(flatten)]
    pub attributes: Map<String, Value>,
}

/// Additional metadata fields can be added to the GeoJSON Object Properties.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Properties {
    /// The searchable date and time of the assets, which must be in UTC.
    ///
    /// It is formatted according to RFC 3339, section 5.6. null is allowed, but
    /// requires `start_datetime` and `end_datetime` from common metadata to be set.
    pub datetime: Option<String>,

    /// Additional fields on the properties.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl Default for Properties {
    fn default() -> Properties {
        Properties {
            datetime: Some(Utc::now().to_rfc3339()),
            additional_fields: Map::new(),
        }
    }
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

    /// Sets this item's collection id in the builder pattern.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// let item = Item::new("an-id").collection("a-collection");
    /// assert_eq!(item.collection.unwrap(), "a-collection");
    pub fn collection(mut self, id: impl ToString) -> Item {
        self.collection = Some(id.to_string());
        self
    }

    /// Returns this item's collection link.
    ///
    /// This is the first link with a rel="collection".
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// let item: Item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.collection_link().unwrap();
    /// ```
    pub fn collection_link(&self) -> Option<&Link> {
        self.links.iter().find(|link| link.is_collection())
    }

    /// Sets this item's geometry.
    ///
    /// Also sets this item's bounding box.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// use geojson::{Geometry, Value};
    ///
    /// let mut item = Item::new("an-id");
    /// item.set_geometry(Some(Geometry::new(Value::Point(vec![-105.1, 41.1]))));
    /// assert_eq!(item.bbox.unwrap(), vec![-105.1, 41.1, -105.1, 41.1]);
    /// ```
    #[cfg(feature = "geo")]
    pub fn set_geometry(&mut self, geometry: impl Into<Option<geojson::Geometry>>) -> Result<()> {
        use geo::BoundingRect;

        let geometry = geometry.into();
        self.bbox = geometry
            .as_ref()
            .and_then(|geometry| geo::Geometry::try_from(geometry).ok())
            .and_then(|geometry| geometry.bounding_rect())
            .map(|rect| {
                let min = rect.min();
                let max = rect.max();
                vec![min.x, min.y, max.x, max.y]
            });
        self.geometry = serde_json::from_value(serde_json::to_value(geometry)?)?;
        Ok(())
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

impl Links for Item {
    fn links(&self) -> &[Link] {
        &self.links
    }
    fn links_mut(&mut self) -> &mut Vec<Link> {
        &mut self.links
    }
}

impl Assets for Item {
    fn assets(&self) -> &HashMap<String, Asset> {
        &self.assets
    }
    fn assets_mut(&mut self) -> &mut HashMap<String, Asset> {
        &mut self.assets
    }
}

impl Extensions for Item {
    fn extensions(&self) -> Option<&[String]> {
        self.extensions.as_deref()
    }
}

impl TryFrom<Item> for Map<String, Value> {
    type Error = Error;
    fn try_from(item: Item) -> Result<Self> {
        if let serde_json::Value::Object(object) = serde_json::to_value(item)? {
            Ok(object)
        } else {
            panic!("all STAC items should serialize to a serde_json::Value::Object")
        }
    }
}

impl TryFrom<Map<String, Value>> for Item {
    type Error = serde_json::Error;
    fn try_from(map: Map<String, Value>) -> std::result::Result<Self, Self::Error> {
        serde_json::from_value(Value::Object(map))
    }
}

fn deserialize_type<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    crate::deserialize_type(deserializer, ITEM_TYPE)
}

fn serialize_type<S>(r#type: &String, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    crate::serialize_type(r#type, serializer, ITEM_TYPE)
}

#[cfg(test)]
mod tests {
    use super::Item;
    use crate::STAC_VERSION;
    use serde_json::Value;

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
    fn deserialize_invalid_type_field() {
        let mut item: Value = crate::read_json("data/simple-item.json").unwrap();
        item["type"] = "Item".into(); // must be "Feature"
        assert!(serde_json::from_value::<Item>(item).is_err());
    }

    #[test]
    fn serialize_invalid_type_field() {
        let mut item = Item::new("an-id");
        item.r#type = "Item".to_string(); // must be "Feature"
        assert!(serde_json::to_value(item).is_err());
    }

    #[test]
    #[cfg(feature = "geo")]
    fn set_geometry_sets_bbox() {
        use geojson::Geometry;
        let mut item = Item::new("an-id");
        item.set_geometry(Some(Geometry::new(geojson::Value::Point(vec![
            -105.1, 41.1,
        ]))))
        .unwrap();
        assert_eq!(item.bbox, Some(vec![-105.1, 41.1, -105.1, 41.1]));
    }

    #[test]
    #[cfg(feature = "geo")]
    fn set_geometry_clears_bbox() {
        use geojson::Geometry;
        let mut item = Item::new("an-id");
        item.set_geometry(Some(Geometry::new(geojson::Value::Point(vec![
            -105.1, 41.1,
        ]))))
        .unwrap();
        item.set_geometry(None).unwrap();
        assert_eq!(item.bbox, None);
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
