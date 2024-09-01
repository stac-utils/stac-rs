use crate::{Error, Extensions, Fields, Href, Link, Links, Migrate, Result, Version, STAC_VERSION};
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
    #[serde(
        deserialize_with = "deserialize_type",
        serialize_with = "serialize_type"
    )]
    r#type: String,

    /// The STAC version the `Catalog` implements.
    #[serde(rename = "stac_version")]
    pub version: Version,

    /// A list of extension identifiers the `Catalog` implements.
    #[serde(rename = "stac_extensions")]
    #[serde(skip_serializing_if = "Vec::is_empty")]
    #[serde(default)]
    pub extensions: Vec<String>,

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

    #[serde(skip)]
    href: Option<String>,
}

impl Catalog {
    /// Creates a new `Catalog` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Catalog;
    /// let catalog = Catalog::new("an-id", "a description");
    /// assert_eq!(catalog.id, "an-id");
    /// assert_eq!(catalog.description, "a description");
    /// ```
    pub fn new(id: impl ToString, description: impl ToString) -> Catalog {
        Catalog {
            r#type: CATALOG_TYPE.to_string(),
            version: STAC_VERSION,
            extensions: Vec::new(),
            id: id.to_string(),
            title: None,
            description: description.to_string(),
            links: Vec::new(),
            additional_fields: Map::new(),
            href: None,
        }
    }
}

impl Href for Catalog {
    fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    fn set_href(&mut self, href: impl ToString) {
        self.href = Some(href.to_string())
    }

    fn clear_href(&mut self) {
        self.href = None;
    }
}

impl Links for Catalog {
    fn links(&self) -> &[Link] {
        &self.links
    }
    fn links_mut(&mut self) -> &mut Vec<Link> {
        &mut self.links
    }
}

impl TryFrom<Catalog> for Map<String, Value> {
    type Error = Error;
    fn try_from(catalog: Catalog) -> Result<Self> {
        if let Value::Object(object) = serde_json::to_value(catalog)? {
            Ok(object)
        } else {
            panic!("all STAC catalogs should serialize to a serde_json::Value::Object")
        }
    }
}

impl TryFrom<Map<String, Value>> for Catalog {
    type Error = serde_json::Error;
    fn try_from(map: Map<String, Value>) -> std::result::Result<Self, Self::Error> {
        serde_json::from_value(Value::Object(map))
    }
}

impl Fields for Catalog {
    fn fields(&self) -> &Map<String, Value> {
        &self.additional_fields
    }
    fn fields_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.additional_fields
    }
}

impl Extensions for Catalog {
    fn extensions(&self) -> &Vec<String> {
        &self.extensions
    }
    fn extensions_mut(&mut self) -> &mut Vec<String> {
        &mut self.extensions
    }
}

fn deserialize_type<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    crate::deserialize_type(deserializer, CATALOG_TYPE)
}

fn serialize_type<S>(r#type: &String, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    crate::serialize_type(r#type, serializer, CATALOG_TYPE)
}

impl Migrate for Catalog {}

#[cfg(test)]
mod tests {
    use super::Catalog;
    use crate::STAC_VERSION;

    #[test]
    fn new() {
        let catalog = Catalog::new("an-id", "a description");
        assert!(catalog.title.is_none());
        assert_eq!(catalog.description, "a description");
        assert_eq!(catalog.r#type, "Catalog");
        assert_eq!(catalog.version, STAC_VERSION);
        assert!(catalog.extensions.is_empty());
        assert_eq!(catalog.id, "an-id");
        assert!(catalog.links.is_empty());
    }

    #[test]
    fn skip_serializing() {
        let catalog = Catalog::new("an-id", "a description");
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
