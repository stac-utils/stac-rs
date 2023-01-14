use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stac::{Collection, Link};

/// Object containing an array of Collection objects in the Catalog, and Link relations.
#[derive(Debug, Serialize, Deserialize)]
pub struct Collections {
    /// The [Collection] objects in the [stac::Catalog].
    pub collections: Vec<Collection>,

    /// The [stac::Link] relations.
    pub links: Vec<Link>,

    /// Additional fields.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl From<Vec<Collection>> for Collections {
    fn from(collections: Vec<Collection>) -> Collections {
        Collections {
            collections,
            links: Vec::new(),
            additional_fields: Map::new(),
        }
    }
}
