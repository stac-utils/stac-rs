use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Crate-specific link type.
///
/// The item search conformance class defines some additional fields on link.
#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    /// The [stac::Link].
    #[serde(flatten)]
    pub link: stac::Link,

    /// The HTTP method of the request, usually GET or POST. Defaults to GET.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// A dictionary of header values that must be included in the next request
    #[serde(skip_serializing_if = "Map::is_empty")]
    pub headers: Map<String, Value>,

    /// If true, the headers/body fields in the next link must be merged into
    /// the original request and be sent combined in the next request. Defaults
    /// to false
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge: Option<bool>,
}

impl TryFrom<stac::Link> for Link {
    type Error = serde_json::Error;

    fn try_from(link: stac::Link) -> Result<Link, Self::Error> {
        serde_json::to_value(link).and_then(serde_json::from_value)
    }
}

impl TryFrom<Link> for stac::Link {
    type Error = serde_json::Error;

    fn try_from(link: Link) -> Result<stac::Link, Self::Error> {
        serde_json::to_value(link).and_then(serde_json::from_value)
    }
}
