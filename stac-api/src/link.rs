use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Crate-specific link type.
///
/// The item search conformance class defines some additional fields on link.
#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    #[serde(flatten)]
    link: stac::Link,

    /// The HTTP method of the request, usually GET or POST. Defaults to GET.
    #[serde(skip_serializing_if = "Option::is_none")]
    method: Option<String>,

    /// A dictionary of header values that must be included in the next request
    #[serde(skip_serializing_if = "Map::is_empty")]
    headers: Map<String, Value>,

    /// If true, the headers/body fields in the next link must be merged into
    /// the original request and be sent combined in the next request. Defaults
    /// to false
    #[serde(skip_serializing_if = "Option::is_none")]
    merge: Option<bool>,
}
