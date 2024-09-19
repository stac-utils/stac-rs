use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stac::{Collection, Href, Link, Links};

/// Object containing an array of collections and an array of links.
#[derive(Debug, Serialize, Deserialize)]
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

impl Href for Collections {
    fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }
    fn set_href(&mut self, href: impl ToString) {
        self.href = Some(href.to_string());
    }
    fn clear_href(&mut self) {
        self.href = None;
    }
}

impl Links for Collections {
    fn links(&self) -> &[Link] {
        &self.links
    }

    fn links_mut(&mut self) -> &mut Vec<Link> {
        &mut self.links
    }
}
