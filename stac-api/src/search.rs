use crate::{Fields, Sortby};
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The core parameters for STAC search are defined by OAFeat, and STAC adds a few parameters for convenience.
#[derive(Debug, Serialize, Deserialize)]
pub struct Search {
    /// The maximum number of results to return (page size).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limit: Option<u64>,

    /// Requested bounding box.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub bbox: Vec<f64>,

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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub ids: Vec<String>,

    /// Array of one or more Collection IDs that each matching Item must be in.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub collections: Vec<String>,

    /// Include/exclude fields from item collections.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<Fields>,

    /// Fields by which to sort results.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sortby: Vec<Sortby>,

    /// `cql2-text` or `cql2-json`.
    ///
    /// If undefined, defaults to `cql2-text` for a GET request and `cql2-json` for a POST request.
    #[serde(skip_serializing_if = "Option::is_none", rename = "filter-lang")]
    pub filter_lang: Option<FilterLang>,

    /// Recommended to not be passed, but server must only accept
    /// <http://www.opengis.net/def/crs/OGC/1.3/CRS84> as a valid value, may
    /// reject any others
    #[serde(skip_serializing_if = "Option::is_none", rename = "filter-crs")]
    pub filter_crs: Option<String>,

    /// CQL2 filter expression.
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub filter: Map<String, Value>,

    /// Additional filtering based on properties.
    ///
    /// It is recommended to use the filter extension instead.
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub query: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum FilterLang {
    /// `cql2-text`
    #[serde(rename = "cql2-text")]
    Cql2Text,

    /// `cql2-json`
    #[serde(rename = "cql2-text")]
    Cql2Json,
}
