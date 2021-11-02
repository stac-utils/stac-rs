use crate::{Link, STAC_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

const CATALOG_TYPE: &str = "Catalog";

#[derive(Debug, Serialize, Deserialize)]
pub struct Catalog {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "stac_version")]
    pub version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub description: String,
    pub links: Vec<Link>,
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl Catalog {
    /// Creates a new `Catalog` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Catalog;
    /// let catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.id, "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Catalog {
        Catalog {
            type_: CATALOG_TYPE.to_string(),
            version: STAC_VERSION.to_string(),
            extensions: None,
            id: id.to_string(),
            title: None,
            description: String::new(),
            links: Vec::new(),
            additional_fields: Map::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Catalog;
    use crate::STAC_VERSION;

    #[test]
    fn new() {
        let catalog = Catalog::new("an-id");
        assert_eq!(catalog.type_, "Catalog");
        assert_eq!(catalog.version, STAC_VERSION);
        assert!(catalog.extensions.is_none());
        assert_eq!(catalog.id, "an-id");
        assert!(catalog.title.is_none());
        assert_eq!(catalog.description, "");
        assert!(catalog.links.is_empty());
        assert!(catalog.additional_fields.is_empty());
    }

    #[test]
    fn skip_serializing() {
        let catalog = Catalog::new("an-id");
        let value = serde_json::to_value(catalog).unwrap();
        assert!(value.get("stac_extensions").is_none());
        assert!(value.get("title").is_none());
    }
    mod roundtrip {
        use super::Catalog;
        use crate::tests::roundtrip;

        roundtrip!(catalog, "examples/catalog.json", Catalog);
    }
}
