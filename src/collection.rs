use crate::{Asset, Extent, Link, Provider, STAC_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// The type field for [Collections](Collection).
pub const COLLECTION_TYPE: &str = "Collection";

/// The STAC `Collection` Specification defines a set of common fields to describe
/// a group of [Items](crate::Item) that share properties and metadata.
///
/// The `Collection` Specification shares all fields with the STAC
/// [Catalog](crate::Catalog) Specification (with different allowed values for
/// `type` and `extensions`) and adds fields to describe the whole dataset and
/// the included set of `Item`s.  `Collection`s can have both parent `Catalogs` and
/// `Collection`s and child `Item`s, `Catalog`s and `Collection`s.
///
/// A STAC `Collection` is represented in JSON format. Any JSON object that
/// contains all the required fields is a valid STAC `Collection` and also a valid
/// STAC `Catalog`.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Collection {
    /// Must be set to `"Collection"` to be a valid `Collection`.
    pub r#type: String,

    /// The STAC version the `Collection` implements.
    #[serde(rename = "stac_version")]
    pub version: String,

    /// A list of extension identifiers the `Collection` implements.
    #[serde(rename = "stac_extensions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Identifier for the `Collection` that is unique across the provider.
    pub id: String,

    /// A short descriptive one-line title for the `Collection`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Detailed multi-line description to fully explain the `Collection`.
    ///
    /// [CommonMark 0.29](http://commonmark.org/) syntax MAY be used for rich text representation.
    pub description: String,

    /// List of keywords describing the `Collection`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub keywords: Option<Vec<String>>,

    /// `Collection`'s license(s), either a SPDX [License
    /// identifier](https://spdx.org/licenses/), `"various"` if multiple licenses
    /// apply or `"proprietary"` for all other cases.
    pub license: String,

    /// A list of [providers](Provider), which may include all organizations capturing or
    /// processing the data or the hosting provider.
    ///
    /// Providers should be listed in chronological order with the most recent
    /// provider being the last element of the list.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub providers: Option<Vec<Provider>>,

    /// Spatial and temporal extents.
    pub extent: Extent,

    /// A map of property summaries, either a set of values, a range of values
    /// or a [JSON Schema](https://json-schema.org).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summaries: Option<Map<String, Value>>,

    /// A list of references to other documents.
    pub links: Vec<Link>,

    /// Dictionary of asset objects that can be downloaded, each with a unique key.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assets: Option<HashMap<String, Asset>>,

    /// Additional fields not part of the `Collection` specification.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl Collection {
    /// Creates a new `Collection` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Collection;
    /// let collection = Collection::new("an-id");
    /// assert_eq!(collection.id, "an-id");
    /// ```
    pub fn new(id: impl ToString) -> Collection {
        Collection {
            r#type: COLLECTION_TYPE.to_string(),
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
        assert!(collection.title.is_none());
        assert_eq!(collection.description, "");
        assert_eq!(collection.license, "");
        assert!(collection.providers.is_none());
        assert_eq!(collection.extent, Extent::default());
        assert!(collection.summaries.is_none());
        assert!(collection.assets.is_none());
        assert_eq!(collection.r#type, "Collection");
        assert_eq!(collection.version, STAC_VERSION);
        assert!(collection.extensions.is_none());
        assert_eq!(collection.id, "an-id");
        assert!(collection.links.is_empty());
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

        roundtrip!(collection, "data/collection.json", Collection);
        roundtrip!(
            collection_with_schemas,
            "data/collection-only/collection-with-schemas.json",
            Collection
        );
        roundtrip!(
            collection_only,
            "data/collection-only/collection.json",
            Collection
        );
        roundtrip!(
            extensions_collection,
            "data/extensions-collection/collection.json",
            Collection
        );
    }
}
