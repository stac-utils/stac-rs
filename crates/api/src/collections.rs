use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stac::{Collection, Link};
use stac_derive::{Href, Links};

/// Object containing an array of collections and an array of links.
#[derive(Debug, Serialize, Deserialize, Href, Links)]
pub struct Collections {
    /// The [Collection] objects in the [stac::Catalog].
    pub collections: Vec<Collection>,

    /// The [stac::Link] relations.
    pub links: Vec<Link>,

    /// Additional fields.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,

    #[serde(skip)]
    href: Option<String>,
}

impl From<Vec<Collection>> for Collections {
    fn from(collections: Vec<Collection>) -> Collections {
        Collections {
            collections,
            links: Vec::new(),
            additional_fields: Map::new(),
            href: None,
        }
    }
}
