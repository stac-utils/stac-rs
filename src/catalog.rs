use crate::core::{Core, CoreStruct};
use serde::{Deserialize, Serialize};

/// The type field for Catalogs.
pub const CATALOG_TYPE: &str = "Catalog";

/// A STAC Catalog object represents a logical group of other Catalog,
/// Collection, and Item objects.
///
/// These Items can be linked to directly from a Catalog, or the Catalog can
/// link to other Catalogs (often called sub-catalogs) that contain links to
/// Collections and Items. The division of sub-catalogs is up to the
/// implementor, but is generally done to aid the ease of online browsing by
/// people.
///
/// A Catalog object will typically be the entry point into a STAC catalog.
/// Their purpose is discovery: to be browsed by people or be crawled by clients
/// to build a searchable index.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Catalog {
    #[serde(flatten)]
    core: CoreStruct,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    description: String,
}

impl Catalog {
    /// Creates a new `Catalog` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Catalog, Core};
    /// let catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.id(), "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Catalog {
        Catalog {
            core: CoreStruct::new(CATALOG_TYPE, id),
            title: None,
            description: String::new(),
        }
    }

    /// Returns a reference to this Catalog's title.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Catalog;
    /// let catalog = Catalog::new("an-id");
    /// assert!(catalog.title().is_none());
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns a reference to this Catalog's description.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Catalog;
    /// let catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.description(), "");
    /// ```
    pub fn description(&self) -> &str {
        &self.description
    }
}

impl AsRef<CoreStruct> for Catalog {
    fn as_ref(&self) -> &CoreStruct {
        &self.core
    }
}

impl AsMut<CoreStruct> for Catalog {
    fn as_mut(&mut self) -> &mut CoreStruct {
        &mut self.core
    }
}

impl Core for Catalog {}

#[cfg(test)]
mod tests {
    use super::Catalog;
    use crate::{Core, STAC_VERSION};

    #[test]
    fn new() {
        let catalog = Catalog::new("an-id");
        assert!(catalog.title().is_none());
        assert_eq!(catalog.description(), "");
        assert_eq!(catalog.type_(), "Catalog");
        assert_eq!(catalog.version(), STAC_VERSION);
        assert!(catalog.extensions().is_none());
        assert_eq!(catalog.id(), "an-id");
        assert!(catalog.links().is_empty());
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
