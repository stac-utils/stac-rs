//! STAC Items.

use crate::{Asset, Assets, Error, Extensions, Fields, Href, Link, Links, Result, STAC_VERSION};
use chrono::{DateTime, FixedOffset, Utc};
use geojson::{feature::Id, Feature, Geometry};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{collections::HashMap, path::Path};
use url::Url;

/// The type field for [Items](Item).
pub const ITEM_TYPE: &str = "Feature";

#[cfg(feature = "wkb")]
const TOP_LEVEL_ATTRIBUTES: [&str; 8] = [
    "type",
    "stac_extensions",
    "id",
    "geometry",
    "bbox",
    "links",
    "assets",
    "collection",
];

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
    #[serde(
        deserialize_with = "deserialize_type",
        serialize_with = "serialize_type"
    )]
    r#type: String,

    /// The STAC version the `Item` implements.
    #[serde(rename = "stac_version")]
    version: String,

    /// A list of extensions the `Item` implements.
    #[serde(rename = "stac_extensions")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub extensions: Vec<String>,

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

/// A [GeoparquetItem] has all of its properties at the top level, and has a
/// Well-Known Binary (WKB) `geometry` field.
///
/// The structure of this item is defined in [the specification](https://github.com/stac-utils/stac-geoparquet/blob/main/spec/stac-geoparquet-spec.md).
#[cfg(feature = "wkb")]
#[derive(Debug, Serialize, Deserialize)]
pub struct GeoparquetItem {
    /// This is just needed for GeoJSON, so it is optional and not recommended
    /// to include in GeoParquet.
    ///
    /// This value is set to `None` by `TryFrom<Item>`.
    pub r#type: Option<String>,

    /// This column is required, but can be empty if no STAC extensions were used
    #[serde(rename = "stac_extensions")]
    pub extensions: Vec<String>,

    /// Required, should be unique within each collection
    pub id: String,

    /// For GeoParquet 1.0 this must be well-known Binary
    pub geometry: Vec<u8>,

    /// Can be a 4 or 6 value struct, depending on dimension of the data.
    ///
    /// It must conform to the "Bounding Box Columns" definition of GeoParquet 1.1.
    pub bbox: Vec<f64>,

    /// List of link objects to resources and related URLs.
    pub links: Vec<Link>,

    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    pub assets: HashMap<String, Asset>,

    /// The ID of the collection this Item is a part of.
    pub collection: Option<String>,

    /// Each property should use the relevant Parquet type, and be pulled out of
    /// the properties object to be a top-level Parquet field
    #[serde(flatten)]
    pub properties: Map<String, Value>,
}

/// Additional metadata fields can be added to the GeoJSON Object Properties.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Properties {
    /// The searchable date and time of the assets, which must be in UTC.
    ///
    /// It is formatted according to RFC 3339, section 5.6. null is allowed, but
    /// requires `start_datetime` and `end_datetime` from common metadata to be set.
    pub datetime: Option<String>,

    /// The first or start date and time for the Item, in UTC.
    ///
    /// It is formatted as date-time according to RFC 3339, section 5.6.
    ///
    /// This is a [common
    /// metadata](https://github.com/radiantearth/stac-spec/blob/master/item-spec/common-metadata.md)
    /// field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_datetime: Option<String>,

    /// The last or end date and time for the Item, in UTC.
    ///
    /// It is formatted as date-time according to RFC 3339, section 5.6.
    ///
    /// This is a [common
    /// metadata](https://github.com/radiantearth/stac-spec/blob/master/item-spec/common-metadata.md)
    /// field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub end_datetime: Option<String>,

    /// A human readable title describing the Item.
    ///
    /// This is a [common
    /// metadata](https://github.com/radiantearth/stac-spec/blob/master/item-spec/common-metadata.md)
    /// field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Detailed multi-line description to fully explain the Item.
    ///
    /// CommonMark 0.29 syntax MAY be used for rich text representation.
    ///
    /// This is a [common
    /// metadata](https://github.com/radiantearth/stac-spec/blob/master/item-spec/common-metadata.md)
    /// field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Creation date and time of the corresponding data, in UTC.
    ///
    /// This identifies the creation time of the metadata.
    ///
    /// This is a [common
    /// metadata](https://github.com/radiantearth/stac-spec/blob/master/item-spec/common-metadata.md)
    /// field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created: Option<String>,

    /// Date and time the metadata was updated last, in UTC.
    ///
    /// This identifies the updated time of the metadata.
    ///
    /// This is a [common
    /// metadata](https://github.com/radiantearth/stac-spec/blob/master/item-spec/common-metadata.md)
    /// field.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated: Option<String>,

    /// Additional fields on the properties.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

/// Builder for a STAC Item.
#[derive(Debug)]
pub struct Builder {
    id: String,
    canonicalize_paths: bool,
    assets: HashMap<String, Asset>,
    enable_gdal: bool,
    #[cfg(feature = "gdal")]
    force_statistics: bool, // TODO add builder method
    #[cfg(feature = "gdal")]
    is_approx_statistics_ok: bool, // TODO add builder method
}

impl Builder {
    /// Creates a new builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::item::Builder;
    /// let builder = Builder::new("an-id");
    /// ```
    pub fn new(id: impl ToString) -> Builder {
        Builder {
            id: id.to_string(),
            canonicalize_paths: true,
            assets: HashMap::new(),
            enable_gdal: cfg!(feature = "gdal"),
            #[cfg(feature = "gdal")]
            force_statistics: false,
            #[cfg(feature = "gdal")]
            is_approx_statistics_ok: true,
        }
    }

    /// Set to false to not canonicalize paths.
    ///
    /// Useful if you want relative paths, or the files don't actually exist.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::item::Builder;
    /// let builder = Builder::new("an-id").canonicalize_paths(false);
    /// ```
    pub fn canonicalize_paths(mut self, canonicalize_paths: bool) -> Builder {
        self.canonicalize_paths = canonicalize_paths;
        self
    }

    /// Adds an asset by href to this builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::item::Builder;
    /// let builder = Builder::new("an-id").asset("data", "assets/dataset.tif");
    /// ```
    pub fn asset(mut self, key: impl ToString, asset: impl Into<Asset>) -> Builder {
        let _ = self.assets.insert(key.to_string(), asset.into());
        self
    }

    /// Enable or disable GDAL processing of asset files.
    ///
    /// If this crate is _not_ compiled with the `gdal` flag and this value is
    /// `true`, an error will be thrown.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::item::Builder;
    /// let builder = Builder::new("an-id").enable_gdal(false);
    /// ```
    pub fn enable_gdal(mut self, enable_gdal: bool) -> Builder {
        self.enable_gdal = enable_gdal;
        self
    }

    /// Creates an [Item] by consuming this builder.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::item::Builder;
    /// let builder = Builder::new("an-id").asset("data", "assets/dataset.tif");
    /// let item = builder.into_item().unwrap();
    /// assert_eq!(item.assets.len(), 1);
    /// ```
    pub fn into_item(self) -> Result<Item> {
        let mut item = Item::new(self.id);
        for (key, mut asset) in self.assets {
            if Url::parse(&asset.href).is_err() && self.canonicalize_paths {
                asset.href = Path::new(&asset.href)
                    .canonicalize()?
                    .to_string_lossy()
                    .into_owned();
            }
            let _ = item.assets.insert(key, asset);
        }
        if self.enable_gdal {
            #[cfg(feature = "gdal")]
            crate::gdal::update_item(
                &mut item,
                self.force_statistics,
                self.is_approx_statistics_ok,
            )?;
            #[cfg(not(feature = "gdal"))]
            return Err(Error::GdalNotEnabled);
        }
        Ok(item)
    }
}

impl Default for Properties {
    fn default() -> Properties {
        Properties {
            datetime: Some(Utc::now().to_rfc3339()),
            start_datetime: None,
            end_datetime: None,
            title: None,
            description: None,
            created: None,
            updated: None,
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
            extensions: Vec::new(),
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
    pub fn set_geometry(&mut self, geometry: impl Into<Option<Geometry>>) -> Result<()> {
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

    /// Returns true if this item's geometry intersects the provided geojson geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// use geojson::{Geometry, Value};
    /// use geo::{Rect, coord};
    ///
    /// let mut item = Item::new("an-id");
    /// item.set_geometry(Some(Geometry::new(Value::Point(vec![-105.1, 41.1]))));
    /// let intersects = Rect::new(
    ///     coord! { x: -106.0, y: 40.0 },
    ///     coord! { x: -105.0, y: 42.0 },
    /// );
    /// assert!(item.intersects(&intersects).unwrap());
    /// ```
    #[cfg(feature = "geo")]
    pub fn intersects<T>(&self, intersects: &T) -> Result<bool>
    where
        T: geo::Intersects<geo::Geometry>,
    {
        if let Some(geometry) = self.geometry.clone() {
            let geometry: geo::Geometry = geometry.try_into()?;
            Ok(intersects.intersects(&geometry))
        } else {
            Ok(false)
        }
    }

    /// Returns true if this item's geometry intersects the provided bounding box.
    ///
    /// DEPRECATED Use `intersects` instead.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// use geojson::{Geometry, Value};
    ///
    /// let mut item = Item::new("an-id");
    /// item.set_geometry(Some(Geometry::new(Value::Point(vec![-105.1, 41.1]))));
    /// let bbox = stac::geo::bbox(&vec![-106.0, 41.0, -105.0, 42.0]).unwrap();
    /// assert!(item.intersects_bbox(bbox).unwrap());
    /// ```
    #[cfg(feature = "geo")]
    #[deprecated(since = "0.5.2", note = "Use intersects instead")]
    pub fn intersects_bbox(&self, bbox: geo::Rect) -> Result<bool> {
        use geo::Intersects;

        if let Some(geometry) = self.geometry.clone() {
            let geometry: geo::Geometry = geometry.try_into()?;
            Ok(geometry.intersects(&bbox))
        } else {
            Ok(false)
        }
    }

    /// Returns true if this item's datetime (or start and end datetime)
    /// intersects the provided datetime string.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// let mut item = Item::new("an-id");
    /// item.properties.datetime = Some("2023-07-11T12:00:00Z".to_string());
    /// assert!(item.intersects_datetime_str("2023-07-11T00:00:00Z/2023-07-12T00:00:00Z").unwrap());
    /// ```
    pub fn intersects_datetime_str(&self, datetime: &str) -> Result<bool> {
        let (start, end) = crate::datetime::parse(datetime)?;
        self.intersects_datetimes(start, end)
    }

    /// Returns true if this item's datetime (or start and end datetimes)
    /// intersects the provided datetime.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// let mut item = Item::new("an-id");
    /// item.properties.datetime = Some("2023-07-11T12:00:00Z".to_string());
    /// let (start, end) = stac::datetime::parse("2023-07-11T00:00:00Z/2023-07-12T00:00:00Z").unwrap();
    /// assert!(item.intersects_datetimes(start, end).unwrap());
    /// ```
    pub fn intersects_datetimes(
        &self,
        start: Option<DateTime<FixedOffset>>,
        end: Option<DateTime<FixedOffset>>,
    ) -> Result<bool> {
        let item_datetime = self
            .properties
            .datetime
            .as_ref()
            .map(|s| DateTime::parse_from_rfc3339(s))
            .transpose()?;
        let item_start = self
            .properties
            .additional_fields
            .get("start_datetime")
            .and_then(|value| value.as_str())
            .map(DateTime::parse_from_rfc3339)
            .transpose()?
            .or(item_datetime);
        let item_end = self
            .properties
            .additional_fields
            .get("end_datetime")
            .and_then(|value| value.as_str())
            .map(DateTime::parse_from_rfc3339)
            .transpose()?
            .or(item_datetime);
        let mut intersects = true;
        if let Some(start) = start {
            if let Some(item_end) = item_end {
                if item_end < start {
                    intersects = false;
                }
            }
        }
        if let Some(end) = end {
            if let Some(item_start) = item_start {
                if item_start > end {
                    intersects = false;
                }
            }
        }
        Ok(intersects)
    }

    /// Converts this item into a [GeoparquetItem].
    ///
    /// If `drop_invalid_attributes` is `True`, any properties that conflict
    /// with top-level field names will be discarded with a warning. If it is
    /// `False`, and error will be raised. The same is true for any top-level
    /// fields that are not part of the spec.
    ///
    /// Raises an error if there's no geometry or bbox set.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// use geojson::{Geometry, Value};
    ///
    /// let mut item = Item::new("an-id");
    /// #[cfg(feature = "wkb")]
    /// {
    /// item.set_geometry(Some(Geometry::new(Value::Point(vec![-105.1, 41.1]))));
    /// let geoparquet_item = item.into_geoparquet_item(true).unwrap();
    /// }
    /// ```
    #[cfg(feature = "wkb")]
    pub fn into_geoparquet_item(self, drop_invalid_attributes: bool) -> Result<GeoparquetItem> {
        let properties = if let Value::Object(object) = serde_json::to_value(self.properties)? {
            object
        } else {
            panic!("properties should always serialize to an object")
        };
        for (key, _) in properties.iter() {
            if TOP_LEVEL_ATTRIBUTES.contains(&key.as_str()) {
                if drop_invalid_attributes {
                    log::warn!("dropping invalid property: {}", key);
                } else {
                    return Err(Error::InvalidAttribute(key.to_string()));
                }
            }
        }
        for (key, _) in self.additional_fields {
            if drop_invalid_attributes {
                log::warn!("dropping out-of-spec top-level attribute: {}", key);
            } else {
                return Err(Error::InvalidAttribute(key));
            }
        }
        Ok(GeoparquetItem {
            r#type: None,
            extensions: self.extensions,
            id: self.id,
            geometry: self
                .geometry
                .map(geo::Geometry::<f64>::try_from)
                .transpose()?
                .map(|geometry| wkb::geom_to_wkb(&geometry))
                .transpose()?
                .ok_or_else(|| Error::MissingGeometry)?,
            bbox: self.bbox.ok_or_else(|| Error::MissingBbox)?,
            links: self.links,
            assets: self.assets,
            collection: self.collection,
            properties,
        })
    }
}

#[cfg(feature = "wkb")]
impl TryFrom<GeoparquetItem> for Item {
    type Error = Error;

    fn try_from(item: GeoparquetItem) -> Result<Item> {
        use std::io::Cursor;

        let geometry = (&wkb::wkb_to_geom(&mut Cursor::new(item.geometry))?).into();
        Ok(Item {
            r#type: item.r#type.unwrap_or_else(|| ITEM_TYPE.to_string()),
            version: STAC_VERSION.to_string(),
            extensions: item.extensions,
            id: item.id,
            geometry: Some(geometry),
            bbox: Some(item.bbox),
            links: item.links,
            assets: item.assets,
            collection: item.collection,
            properties: serde_json::from_value(item.properties.into())?,
            additional_fields: Default::default(),
            href: None,
        })
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

impl Fields for Item {
    fn fields(&self) -> &Map<String, Value> {
        &self.properties.additional_fields
    }
    fn fields_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.properties.additional_fields
    }
}

impl Extensions for Item {
    fn extensions(&self) -> &Vec<String> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut Vec<String> {
        &mut self.extensions
    }
}

impl TryFrom<Item> for Map<String, Value> {
    type Error = Error;
    fn try_from(item: Item) -> Result<Self> {
        if let Value::Object(object) = serde_json::to_value(item)? {
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

impl TryFrom<Feature> for Item {
    type Error = Error;

    fn try_from(feature: Feature) -> Result<Item> {
        if let Some(id) = feature.id {
            let mut item = Item::new(match id {
                Id::String(id) => id,
                Id::Number(id) => id.to_string(),
            });
            item.bbox = feature.bbox;
            item.geometry = feature.geometry;
            item.properties = feature
                .properties
                .map(|properties| serde_json::from_value::<Properties>(Value::Object(properties)))
                .transpose()?
                .unwrap_or_default();
            item.additional_fields = feature.foreign_members.unwrap_or_default();
            Ok(item)
        } else {
            Err(Error::MissingId)
        }
    }
}

impl TryFrom<Item> for Feature {
    type Error = Error;
    fn try_from(item: Item) -> Result<Feature> {
        Ok(Feature {
            bbox: item.bbox,
            geometry: item.geometry,
            id: Some(Id::String(item.id)),
            properties: match serde_json::to_value(item.properties)? {
                Value::Object(object) => Some(object),
                _ => panic!("properties should always serialize to an object"),
            },
            foreign_members: if item.additional_fields.is_empty() {
                None
            } else {
                Some(item.additional_fields)
            },
        })
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
    use super::{Builder, Item};
    use crate::{extensions::Raster, Asset, Extensions, STAC_VERSION};
    use geojson::{feature::Id, Feature};
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
        assert!(item.extensions.is_empty());
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

    #[test]
    #[cfg(feature = "geo")]
    fn insersects() {
        use geojson::Geometry;
        let mut item = Item::new("an-id");
        item.set_geometry(Some(Geometry::new(geojson::Value::Point(vec![
            -105.1, 41.1,
        ]))))
        .unwrap();
        assert!(item
            .intersects(&crate::geo::bbox(&vec![-106.0, 41.0, -105.0, 42.0]).unwrap())
            .unwrap());
    }

    #[test]
    fn intersects_datetime() {
        let mut item = Item::new("an-id");
        item.properties.datetime = Some("2023-07-11T12:00:00Z".to_string());
        for datetime in [
            "2023-07-11T12:00:00Z",
            "2023-07-11T00:00:00Z/2023-07-12T00:00:00Z",
            "../2023-07-12T00:00:00Z",
            "2023-07-11T00:00:00Z/..",
        ] {
            let (start, end) = crate::datetime::parse(datetime).unwrap();
            assert!(item.intersects_datetimes(start, end).unwrap());
        }
        let (start, end) =
            crate::datetime::parse("2023-07-12T00:00:00Z/2023-07-13T00:00:00Z").unwrap();
        assert!(!item.intersects_datetimes(start, end).unwrap());
        item.properties.datetime = None;
        let _ = item
            .properties
            .additional_fields
            .insert("start_datetime".to_string(), "2023-07-11T11:00:00Z".into());
        let _ = item
            .properties
            .additional_fields
            .insert("end_datetime".to_string(), "2023-07-11T13:00:00Z".into());
        let (start, end) = crate::datetime::parse("2023-07-11T12:00:00Z").unwrap();
        assert!(item.intersects_datetimes(start, end).unwrap());
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

    #[test]
    fn builder() {
        let builder = Builder::new("an-id").asset("data", "assets/dataset.tif");
        let item = builder.into_item().unwrap();
        assert_eq!(item.assets.len(), 1);
        let asset = item.assets.get("data").unwrap();
        assert!(asset
            .href
            .ends_with(&format!("assets{}dataset.tif", std::path::MAIN_SEPARATOR)));
    }

    #[test]
    fn builder_relative_paths() {
        let builder = Builder::new("an-id")
            .canonicalize_paths(false)
            .asset("data", "assets/dataset.tif");
        let item = builder.into_item().unwrap();
        let asset = item.assets.get("data").unwrap();
        assert_eq!(asset.href, "assets/dataset.tif");
    }

    #[test]
    fn builder_asset_roles() {
        let item = Builder::new("an-id")
            .asset("data", Asset::new("assets/dataset.tif").role("data"))
            .into_item()
            .unwrap();
        let asset = item.assets.get("data").unwrap();
        assert_eq!(asset.roles, vec!["data"]);
    }

    #[test]
    fn builder_uses_gdal() {
        let item = Builder::new("an-id")
            .asset("data", "assets/dataset.tif")
            .into_item()
            .unwrap();
        if cfg!(feature = "gdal") {
            assert!(item.has_extension::<Raster>());
        } else {
            assert!(!item.has_extension::<Raster>());
        }
    }

    #[test]
    fn try_from_geojson_feature() {
        let mut feature = Feature {
            bbox: None,
            geometry: None,
            id: None,
            properties: None,
            foreign_members: None,
        };
        let _ = Item::try_from(feature.clone()).unwrap_err();
        feature.id = Some(Id::String("an-id".to_string()));
        let _ = Item::try_from(feature).unwrap();
    }

    #[test]
    fn try_into_geojson_feature() {
        let item = Item::new("an-id");
        let feature = Feature::try_from(item).unwrap();
        assert_eq!(feature.id.unwrap(), Id::String("an-id".to_string()));
    }

    #[test]
    #[cfg(feature = "wkb")]
    fn item_into_geoparquet_item() {
        use geojson::Geometry;

        let mut item = Item::new("an-id");
        let _ = item.clone().into_geoparquet_item(true).unwrap_err(); // no geometry

        let geometry = Geometry::new(geojson::Value::Point(vec![-105.1, 41.1]));
        item.geometry = Some(geometry.clone());
        let _ = item.clone().into_geoparquet_item(true).unwrap_err(); // no bbox

        item.set_geometry(geometry).unwrap();
        let _ = item.clone().into_geoparquet_item(true).unwrap();
        let _ = item.clone().into_geoparquet_item(false).unwrap();

        let _ = item
            .properties
            .additional_fields
            .insert("bbox".to_string(), vec![42.0].into());
        let _ = item.clone().into_geoparquet_item(true).unwrap();
        let _ = item.clone().into_geoparquet_item(false).unwrap_err();

        item.properties.additional_fields = Default::default();
        let _ = item
            .additional_fields
            .insert("foo".to_string(), "bar".to_string().into());
        let _ = item.clone().into_geoparquet_item(true).unwrap();
        let _ = item.clone().into_geoparquet_item(false).unwrap_err();
    }

    #[test]
    #[cfg(feature = "wkb")]
    fn geoparquet_item_into_item() {
        use super::GeoparquetItem;

        let geoparquet_item = GeoparquetItem {
            r#type: None,
            extensions: Vec::new(),
            id: "an-id".to_string(),
            geometry: wkb::geom_to_wkb(&geo::Geometry::Point((-105., 41.).into())).unwrap(),
            bbox: vec![-105., 41., -105., 41.],
            links: Vec::new(),
            assets: Default::default(),
            collection: None,
            properties: Default::default(),
        };
        let _ = Item::try_from(geoparquet_item).unwrap();
    }
}
