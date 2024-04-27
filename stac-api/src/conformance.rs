use serde::{Deserialize, Serialize};

/// The core conformance uri.
pub const CORE_URI: &str = "https://api.stacspec.org/v1.0.0/core";

/// The features conformance uri.
pub const FEATURES_URI: &str = "https://api.stacspec.org/v1.0.0/ogcapi-features";

/// The collections conformance uri.
pub const COLLECTIONS_URI: &str = "https://api.stacspec.org/v1.0.0/collections";

/// The OGC API - Features - Part 1 Requirements Class Core uri
pub const OGC_API_FEATURES_URI: &str =
    "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/core";

/// The GeoJSON spec conformance uri.
pub const GEOJSON_URI: &str = "http://www.opengis.net/spec/ogcapi-features-1/1.0/conf/geojson";

/// The item search conformance uri.
pub const ITEM_SEARCH_URI: &str = "https://api.stacspec.org/v1.0.0/item-search";

/// To support "generic" clients that want to access multiple OGC API Features
/// implementations - and not "just" a specific API / server, the server has to
/// declare the conformance classes it implements and conforms to.
#[derive(Debug, Serialize, Deserialize)]
pub struct Conformance {
    /// The conformance classes it implements and conforms to.
    #[serde(rename = "conformsTo")]
    pub conforms_to: Vec<String>,
}
