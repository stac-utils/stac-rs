use crate::{Link, STAC_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The type field for [Catalogs](Catalog).
pub const CATALOG_TYPE: &str = "Catalog";

/// A STAC Catalog object represents a logical group of other `Catalog`,
/// [Collection](crate::Collection), and [Item](crate::Item) objects.
///
/// These `Item`s can be linked to directly from a `Catalog`, or the `Catalog`
/// can link to other Catalogs (often called sub-catalogs) that contain links to
/// `Collection`s and `Item`s. The division of sub-catalogs is up to the
/// implementor, but is generally done to aid the ease of online browsing by
/// people.
///
/// A `Catalog` object will typically be the entry point into a STAC catalog.
/// Their purpose is discovery: to be browsed by people or be crawled by clients
/// to build a searchable index.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Catalog {
    /// Set to `"Catalog"` if this Catalog only implements the `Catalog` spec.
    pub r#type: String,

    /// The STAC version the `Catalog` implements.
    #[serde(rename = "stac_version")]
    pub version: String,

    /// A list of extension identifiers the `Catalog` implements.
    #[serde(rename = "stac_extensions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,

    /// Identifier for the `Catalog`.
    pub id: String,

    /// A short descriptive one-line title for the `Catalog`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Detailed multi-line description to fully explain the `Catalog`.
    ///
    /// [CommonMark 0.29](http://commonmark.org/) syntax MAY be used for rich text representation.
    pub description: String,

    /// A list of references to other documents.
    pub links: Vec<Link>,

    /// Additional fields not part of the Catalog specification.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl Catalog {
    /// Creates a new `Catalog` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Catalog;
    /// let catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.id, "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Catalog {
        Catalog {
            r#type: CATALOG_TYPE.to_string(),
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
        assert!(catalog.title.is_none());
        assert_eq!(catalog.description, "");
        assert_eq!(catalog.r#type, "Catalog");
        assert_eq!(catalog.version, STAC_VERSION);
        assert!(catalog.extensions.is_none());
        assert_eq!(catalog.id, "an-id");
        assert!(catalog.links.is_empty());
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

        roundtrip!(catalog, "data/catalog.json", Catalog);
    }
}
