use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// Additional metadata fields can be added to the GeoJSON Object Properties.
#[derive(Debug, Serialize, Deserialize)]
pub struct Properties {
    /// The searchable date and time of the assets, which must be in UTC.
    ///
    /// It is formatted according to RFC 3339, section 5.6. null is allowed, but
    /// requires start_datetime and end_datetime from common metadata to be set.
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
