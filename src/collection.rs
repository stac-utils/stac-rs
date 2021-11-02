use crate::{Asset, Extent, Link, Provider, STAC_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

const COLLECTION_TYPE: &str = "Collection";

#[derive(Debug, Serialize, Deserialize)]
pub struct Collection {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "stac_version")]
    pub version: String,
    #[serde(rename = "stac_extensions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,
    pub license: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<Provider>>,
    pub extent: Extent,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summaries: Option<Map<String, Value>>,
    pub links: Vec<Link>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<HashMap<String, Asset>>,
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl Collection {
    /// Creates a new `Collection` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Collection;
    /// let collection = Collection::new("an-id");
    /// assert_eq!(collection.id, "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Collection {
        Collection {
            type_: COLLECTION_TYPE.to_string(),
            version: STAC_VERSION.to_string(),
            extensions: None,
            id: id.to_string(),
            title: None,
            description: String::new(),
            keywords: None,
            license: String::new(),
            providers: None,
            extent: Extent::default(),
            summaries: None,
            links: Vec::new(),
            assets: None,
            additional_fields: Map::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Collection;
    use crate::{Extent, STAC_VERSION};

    #[test]
    fn new() {
        let collection = Collection::new("an-id");
        assert_eq!(collection.type_, "Collection");
        assert_eq!(collection.version, STAC_VERSION);
        assert!(collection.extensions.is_none());
        assert_eq!(collection.id, "an-id");
        assert!(collection.title.is_none());
        assert_eq!(collection.description, "");
        assert_eq!(collection.license, "");
        assert!(collection.providers.is_none());
        assert_eq!(collection.extent, Extent::default());
        assert!(collection.summaries.is_none());
        assert!(collection.links.is_empty());
        assert!(collection.assets.is_none());
        assert!(collection.additional_fields.is_empty());
    }

    #[test]
    fn skip_serializing() {
        let collection = Collection::new("an-id");
        let value = serde_json::to_value(collection).unwrap();
        assert!(value.get("stac_extensions").is_none());
        assert!(value.get("title").is_none());
        assert!(value.get("keywords").is_none());
        assert!(value.get("providers").is_none());
        assert!(value.get("summaries").is_none());
        assert!(value.get("assets").is_none());
    }
    mod roundtrip {
        use super::Collection;
        use crate::tests::roundtrip;

        roundtrip!(collection, "examples/collection.json", Collection);
        roundtrip!(
            collection_with_schemas,
            "examples/collection-only/collection-with-schemas.json",
            Collection
        );
        roundtrip!(
            collection_only,
            "examples/collection-only/collection.json",
            Collection
        );
        roundtrip!(
            extensions_collection,
            "examples/extensions-collection/collection.json",
            Collection
        );
    }
}
