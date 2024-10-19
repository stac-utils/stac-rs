use std::ops::{Deref, DerefMut};

use crate::{Error, GetItems, Items, Result};
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stac::Item;

/// The core parameters for STAC search are defined by OAFeat, and STAC adds a few parameters for convenience.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Search {
    /// Many fields are shared with [Items], so we re-use that structure.
    #[serde(flatten)]
    pub items: Items,

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
}

/// GET parameters for the item search endpoint.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GetSearch {
    /// Many fields are shared with [Items], so we re-use that structure.
    #[serde(flatten)]
    pub items: GetItems,

    /// Searches items by performing intersection between their geometry and provided GeoJSON geometry.
    ///
    /// All GeoJSON geometry types must be supported.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intersects: Option<String>,

    /// Comma-delimited list of Item ids to return.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ids: Option<String>,

    /// Comma-delimited list of one or more Collection IDs that each matching Item must be in.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collections: Option<String>,
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

    /// Returns an error if this search is invalid, e.g. if both bbox and intersects are specified.
    ///
    /// Returns the search unchanged if it is valid.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// use geojson::{Geometry, Value};
    ///
    /// let mut search = Search::default();
    /// search.items.bbox =  Some(vec![-180.0, -90.0, 180.0, 80.0].try_into().unwrap());
    /// search = search.valid().unwrap();
    /// search.intersects = Some(Geometry::new(Value::Point(vec![0.0, 0.0])));
    /// search.valid().unwrap_err();
    /// ```
    pub fn valid(mut self) -> Result<Search> {
        self.items = self.items.valid()?;
        if self.items.bbox.is_some() & self.intersects.is_some() {
            Err(Error::SearchHasBboxAndIntersects(self.clone()))
        } else {
            Ok(self)
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
    pub fn matches(&self, item: &Item) -> Result<bool> {
        Ok(self.collection_matches(item)
            & self.id_matches(item)
            & self.intersects_matches(item)?
            & self.items.matches(item)?)
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
    /// search.intersects = Some(Geometry::new(Value::Point(vec![-105.1, 41.1])));
    /// assert!(!search.intersects_matches(&item).unwrap());
    /// item.set_geometry(Geometry::new(Value::Point(vec![-105.1, 41.1])));
    /// assert!(search.intersects_matches(&item).unwrap());
    /// # }
    /// ```
    #[allow(unused_variables)]
    pub fn intersects_matches(&self, item: &Item) -> Result<bool> {
        if let Some(intersects) = self.intersects.clone() {
            #[cfg(feature = "geo")]
            {
                let intersects: geo::Geometry = intersects.try_into()?;
                item.intersects(&intersects).map_err(Error::from)
            }
            #[cfg(not(feature = "geo"))]
            {
                Err(Error::FeatureNotEnabled("geo"))
            }
        } else {
            Ok(true)
        }
    }
}

impl TryFrom<Search> for GetSearch {
    type Error = Error;

    fn try_from(search: Search) -> Result<GetSearch> {
        let get_items: GetItems = search.items.try_into()?;
        let intersects = search
            .intersects
            .map(|intersects| serde_json::to_string(&intersects))
            .transpose()?;
        let collections = search.collections.map(|collections| collections.join(","));
        let ids = search.ids.map(|ids| ids.join(","));
        Ok(GetSearch {
            items: get_items,
            intersects,
            ids,
            collections,
        })
    }
}

impl TryFrom<GetSearch> for Search {
    type Error = Error;

    fn try_from(get_search: GetSearch) -> Result<Search> {
        let items: Items = get_search.items.try_into()?;
        let intersects = get_search
            .intersects
            .map(|intersects| serde_json::from_str(&intersects))
            .transpose()?;
        let collections = get_search
            .collections
            .map(|collections| collections.split(',').map(|s| s.to_string()).collect());
        let ids = get_search
            .ids
            .map(|ids| ids.split(',').map(|s| s.to_string()).collect());
        Ok(Search {
            items,
            intersects,
            ids,
            collections,
        })
    }
}

impl From<Items> for Search {
    fn from(items: Items) -> Self {
        Search {
            items,
            ..Default::default()
        }
    }
}

impl stac::Fields for Search {
    fn fields(&self) -> &Map<String, Value> {
        &self.items.additional_fields
    }
    fn fields_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.items.additional_fields
    }
}

impl Deref for Search {
    type Target = Items;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl DerefMut for Search {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.items
    }
}
