use crate::{Error, Fields, Filter, Result, Sortby};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

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

/// GET parameters for the items endpoint from STAC API - Features.
///
/// This is a lot like [Search](crate::Search), but without intersects, ids, and
/// collections.
#[derive(Clone, Default, Debug, Serialize, Deserialize)]
pub struct GetItems {
    /// The maximum number of results to return (page size).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<String>,

    /// Requested bounding box.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<String>,

    /// Single date+time, or a range ('/' separator), formatted to [RFC 3339,
    /// section 5.6](https://tools.ietf.org/html/rfc3339#section-5.6).
    ///
    /// Use double dots `..` for open date ranges.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub datetime: Option<String>,

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

impl Items {
    /// Converts this items structure into a [GetItems].
    ///
    /// Used as query parameters in a GET request.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::{Items, GetItems, Fields};
    /// let items = Items {
    ///     fields: Some(Fields {
    ///         include: vec!["foo".to_string()],
    ///         exclude: vec!["bar".to_string()],
    ///     }),
    ///     ..Default::default()
    /// };
    /// let get_items = items.into_get_items().unwrap();
    /// assert_eq!(get_items.fields.unwrap(), "foo,-bar");
    pub fn into_get_items(self) -> Result<GetItems> {
        if let Some(query) = self.query {
            return Err(Error::CannotConvertQueryToString(query));
        }
        let filter = if let Some(filter) = self.filter {
            match filter {
                Filter::Cql2Json(json) => return Err(Error::CannotConvertCql2JsonToString(json)),
                Filter::Cql2Text(text) => Some(text),
            }
        } else {
            None
        };
        Ok(GetItems {
            limit: self.limit.map(|n| n.to_string()),
            bbox: self.bbox.map(|bbox| {
                bbox.into_iter()
                    .map(|n| n.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }),
            datetime: self.datetime,
            fields: self.fields.map(|fields| fields.to_string()),
            sortby: self.sortby.map(|sortby| {
                sortby
                    .into_iter()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
                    .join(",")
            }),
            filter_crs: self.filter_crs,
            filter_lang: filter.as_ref().map(|_| "cql2-text".to_string()),
            filter: filter,
            additional_fields: self
                .additional_fields
                .into_iter()
                .map(|(key, value)| (key, value.to_string()))
                .collect(),
        })
    }
}
