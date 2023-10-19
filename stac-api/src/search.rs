use crate::{Error, Fields, Filter, GetItems, Items, Result, Sortby};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stac::{Geometry, Item};
use std::collections::HashMap;

/// The core parameters for STAC search are defined by OAFeat, and STAC adds a few parameters for convenience.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Search {
    /// The maximum number of results to return (page size).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// Requested bounding box.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<f64>>,

    /// Single date+time, or a range ('/' separator), formatted to [RFC 3339,
    /// section 5.6](https://tools.ietf.org/html/rfc3339#section-5.6).
    ///
    /// Use double dots `..` for open date ranges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,

    /// Searches items by performing intersection between their geometry and provided GeoJSON geometry.
    ///
    /// All GeoJSON geometry types must be supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intersects: Option<Geometry>,

    /// Array of Item ids to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,

    /// Array of one or more Collection IDs that each matching Item must be in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collections: Option<Vec<String>>,

    /// Include/exclude fields from item collections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Fields>,

    /// Fields by which to sort results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortby: Option<Vec<Sortby>>,

    /// Recommended to not be passed, but server must only accept
    /// <http://www.opengis.net/def/crs/OGC/1.3/CRS84> as a valid value, may
    /// reject any others
    #[serde(skip_serializing_if = "Option::is_none", rename = "filter-crs")]
    pub filter_crs: Option<String>,

    /// CQL2 filter expression.
    #[serde(flatten, skip_serializing_if = "Option::is_none")]
    pub filter: Option<Filter>,

    /// Additional filtering based on properties.
    ///
    /// It is recommended to use the filter extension instead.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query: Option<Map<String, Value>>,

    /// Additional fields.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

/// GET parameters for the item search endpoint.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct GetSearch {
    /// The maximum number of results to return (page size).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,

    /// Requested bounding box.
    pub bbox: Option<String>,

    /// Requested bounding box.
    /// Use double dots `..` for open date ranges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,

    /// Searches items by performing intersection between their geometry and provided GeoJSON geometry.
    ///
    /// All GeoJSON geometry types must be supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intersects: Option<String>,

    /// Array of Item ids to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<Vec<String>>,

    /// Array of one or more Collection IDs that each matching Item must be in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collections: Option<Vec<String>>,

    /// Include/exclude fields from item collections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<String>,

    /// Fields by which to sort results.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sortby: Option<String>,

    /// Recommended to not be passed, but server must only accept
    /// <http://www.opengis.net/def/crs/OGC/1.3/CRS84> as a valid value, may
    /// reject any others
    #[serde(skip_serializing_if = "Option::is_none", rename = "filter-crs")]
    pub filter_crs: Option<String>,

    /// CQL2 filter expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter_lang: Option<String>,

    /// CQL2 filter expression.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,

    /// Additional fields.
    #[serde(flatten)]
    pub additional_fields: HashMap<String, String>,
}

impl Search {
    /// Creates a new, empty search.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    ///
    /// let search = Search::new();
    /// ```
    pub fn new() -> Search {
        Search::default()
    }

    /// Sets the ids field of this search.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// let search = Search::new().ids(vec!["an-id".to_string()]);
    /// ```
    pub fn ids(mut self, ids: impl Into<Option<Vec<String>>>) -> Search {
        self.ids = ids.into();
        self
    }

    /// Validates this search.
    ///
    /// E.g. the search is invalid if both bbox and intersects are specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// let mut search = Search { bbox: Some(vec![-180.0, -90.0, 180.0, 80.0]), ..Default::default() };
    /// search.validate().unwrap();
    /// search.intersects = Some(stac::Geometry::point(0., 0.));
    /// let _ = search.validate().unwrap_err();
    /// ```
    pub fn validate(&self) -> Result<()> {
        if self.bbox.is_some() & self.intersects.is_some() {
            Err(Error::SearchHasBboxAndIntersects(self.clone()))
        } else {
            Ok(())
        }
    }

    /// Returns true if this item matches this search.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// use stac_api::Search;
    ///
    /// let item = Item::new("an-id");
    /// assert!(Search::new().matches(&item).unwrap());
    /// assert!(!Search::new().ids(vec!["not-the-id".to_string()]).matches(&item).unwrap());
    /// ```
    #[cfg(feature = "geo")]
    pub fn matches(&self, item: &Item) -> Result<bool> {
        Ok(self.collection_matches(item)
            & self.id_matches(item)
            & self.bbox_matches(item)?
            & self.intersects_matches(item)?
            & self.datetime_matches(item)?
            & self.query_matches(item)?
            & self.filter_matches(item)?)
    }

    /// Returns true if this item's collection matches this search.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// use stac::Item;
    ///
    /// let mut search = Search::new();
    /// let mut item = Item::new("item-id");
    /// assert!(search.collection_matches(&item));
    /// search.collections = Some(vec!["collection-id".to_string()]);
    /// assert!(!search.collection_matches(&item));
    /// item.collection = Some("collection-id".to_string());
    /// assert!(search.collection_matches(&item));
    /// item.collection = Some("another-collection-id".to_string());
    /// assert!(!search.collection_matches(&item));
    /// ```
    pub fn collection_matches(&self, item: &Item) -> bool {
        if let Some(collections) = self.collections.as_ref() {
            if let Some(collection) = item.collection.as_ref() {
                collections.contains(collection)
            } else {
                false
            }
        } else {
            true
        }
    }

    /// Returns true if this item's id matches this search.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// use stac::Item;
    ///
    /// let mut search = Search::new();
    /// let mut item = Item::new("item-id");
    /// assert!(search.id_matches(&item));
    /// search.ids = Some(vec!["item-id".to_string()]);
    /// assert!(search.id_matches(&item));
    /// search.ids = Some(vec!["another-id".to_string()]);
    /// assert!(!search.id_matches(&item));
    /// ```
    pub fn id_matches(&self, item: &Item) -> bool {
        if let Some(ids) = self.ids.as_ref() {
            ids.contains(&item.id)
        } else {
            true
        }
    }

    /// Returns true if this item's geometry matches this search's bbox.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "geo")]
    /// # {
    /// use stac_api::Search;
    /// use stac::Item;
    /// use geojson::{Geometry, Value};
    ///
    /// let mut search = Search::new();
    /// let mut item = Item::new("item-id");
    /// assert!(search.bbox_matches(&item).unwrap());
    /// search.bbox = Some(vec![-110.0, 40.0, -100.0, 50.0]);
    /// assert!(!search.bbox_matches(&item).unwrap());
    /// item.set_geometry(Geometry::new(Value::Point(vec![-105.1, 41.1])));
    /// assert!(search.bbox_matches(&item).unwrap());
    /// # }
    /// ```
    #[cfg(feature = "geo")]
    pub fn bbox_matches(&self, item: &Item) -> Result<bool> {
        if let Some(bbox) = self.bbox.as_ref() {
            let bbox = stac::geo::bbox(bbox)?;
            item.intersects(&bbox).map_err(Error::from)
        } else {
            Ok(true)
        }
    }

    /// Returns true if this item's geometry matches this search's intersects.
    ///
    /// # Examples
    ///
    /// ```
    /// # #[cfg(feature = "geo")]
    /// # {
    /// use stac_api::Search;
    /// use stac::Item;
    /// use geojson::{Geometry, Value};
    ///
    /// let mut search = Search::new();
    /// let mut item = Item::new("item-id");
    /// assert!(search.intersects_matches(&item).unwrap());
    /// search.intersects = Some(stac::Geometry::point(-105.1, 41.1));
    /// assert!(!search.intersects_matches(&item).unwrap());
    /// item.set_geometry(Geometry::new(Value::Point(vec![-105.1, 41.1])));
    /// assert!(search.intersects_matches(&item).unwrap());
    /// # }
    /// ```
    #[cfg(feature = "geo")]
    pub fn intersects_matches(&self, item: &Item) -> Result<bool> {
        if let Some(intersects) = self.intersects.clone() {
            let intersects: geo::Geometry = intersects.try_into()?;
            item.intersects(&intersects).map_err(Error::from)
        } else {
            Ok(true)
        }
    }

    /// Returns true if this item's datetime matches this search.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// use stac::Item;
    ///
    /// let mut search = Search::new();
    /// let mut item = Item::new("item-id");  // default datetime is now
    /// assert!(search.datetime_matches(&item).unwrap());
    /// search.datetime = Some("../2023-10-09T00:00:00Z".to_string());
    /// assert!(!search.datetime_matches(&item).unwrap());
    /// item.properties.datetime = Some("2023-10-08T00:00:00Z".to_string());
    /// assert!(search.datetime_matches(&item).unwrap());
    /// ```
    pub fn datetime_matches(&self, item: &Item) -> Result<bool> {
        if let Some(datetime) = self.datetime.as_ref() {
            item.intersects_datetime_str(datetime).map_err(Error::from)
        } else {
            Ok(true)
        }
    }

    /// Returns true if this item's matches this search query.
    ///
    /// Currently unsupported, always raises an error if query is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// use stac::Item;
    ///
    /// let mut search = Search::new();
    /// let mut item = Item::new("item-id");
    /// assert!(search.query_matches(&item).unwrap());
    /// search.query = Some(Default::default());
    /// assert!(search.query_matches(&item).is_err());
    /// ```
    pub fn query_matches(&self, _: &Item) -> Result<bool> {
        if let Some(_) = self.query.as_ref() {
            // TODO implement
            Err(Error::Unimplemented("query"))
        } else {
            Ok(true)
        }
    }

    /// Returns true if this item matches this search's filter.
    ///
    /// Currently unsupported, always raises an error if filter is set.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// use stac::Item;
    ///
    /// let mut search = Search::new();
    /// let mut item = Item::new("item-id");
    /// assert!(search.filter_matches(&item).unwrap());
    /// search.filter = Some(Default::default());
    /// assert!(search.filter_matches(&item).is_err());
    /// ```
    pub fn filter_matches(&self, _: &Item) -> Result<bool> {
        if let Some(_) = self.filter.as_ref() {
            // TODO implement
            Err(Error::Unimplemented("filter"))
        } else {
            Ok(true)
        }
    }
}

impl TryFrom<Search> for GetSearch {
    type Error = Error;

    fn try_from(search: Search) -> Result<GetSearch> {
        let items = Items {
            limit: search.limit,
            bbox: search.bbox,
            datetime: search.datetime,
            fields: search.fields,
            sortby: search.sortby,
            filter_crs: search.filter_crs,
            filter: search.filter,
            query: search.query,
            additional_fields: search.additional_fields,
        };
        let get_items: GetItems = items.try_into()?;
        let intersects = search
            .intersects
            .map(|intersects| serde_json::to_string(&intersects))
            .transpose()?;
        Ok(GetSearch {
            limit: get_items.limit,
            bbox: get_items.bbox,
            datetime: get_items.datetime,
            intersects: intersects,
            ids: search.ids,
            collections: search.collections,
            fields: get_items.fields,
            sortby: get_items.sortby,
            filter_crs: get_items.filter_crs,
            filter_lang: get_items.filter_lang,
            filter: get_items.filter,
            additional_fields: get_items.additional_fields,
        })
    }
}

impl TryFrom<GetSearch> for Search {
    type Error = Error;

    fn try_from(get_search: GetSearch) -> Result<Search> {
        let get_items = GetItems {
            limit: get_search.limit,
            bbox: get_search.bbox,
            datetime: get_search.datetime,
            fields: get_search.fields,
            sortby: get_search.sortby,
            filter_crs: get_search.filter_crs,
            filter: get_search.filter,
            filter_lang: get_search.filter_lang,
            additional_fields: get_search.additional_fields,
        };
        let items: Items = get_items.try_into()?;
        let intersects = get_search
            .intersects
            .map(|intersects| serde_json::from_str(&intersects))
            .transpose()?;
        Ok(Search {
            limit: items.limit,
            bbox: items.bbox,
            datetime: items.datetime,
            intersects: intersects,
            ids: get_search.ids,
            collections: get_search.collections,
            fields: items.fields,
            sortby: items.sortby,
            filter_crs: items.filter_crs,
            filter: items.filter,
            query: items.query,
            additional_fields: items.additional_fields,
        })
    }
}
