use crate::{Fields, Filter, Sortby};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Parameters for the items endpoint from STAC API - Features.
///
/// This is a lot like [Search](crate::Search), but without intersects, ids, and
/// collections.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct Items {
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
    #[serde(skip_serializing_if = "Option::is_none")]
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

impl Items {
    /// Creates a new items filter.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::Items;
    /// let items = Items::new();
    /// ```
    pub fn new() -> Items {
        Default::default()
    }

    /// Sets this items' limit.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::Items;
    /// let items = Items::new().limit(42);
    /// ```
    pub fn limit(mut self, limit: impl Into<Option<u64>>) -> Items {
        self.limit = limit.into();
        self
    }

    /// Sets this search's fields.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::Items;
    /// let fields = "+foo,-bar".parse().unwrap();
    /// let items = Items::new().fields(Some(fields));
    /// ```
    pub fn fields(mut self, fields: impl Into<Option<Fields>>) -> Items {
        self.fields = fields.into();
        self
    }
}
