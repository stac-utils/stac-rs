use crate::{
    core::{Core, CoreStruct},
    Asset, Extent, Provider,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// The type field for Collections.
pub const COLLECTION_TYPE: &str = "Collection";

/// The STAC Collection Specification defines a set of common fields to describe
/// a group of Items that share properties and metadata.
///
/// The Collection Specification shares all fields with the STAC Catalog
/// Specification (with different allowed values for type and stac_extensions)
/// and adds fields to describe the whole dataset and the included set of Items.
/// Collections can have both parent Catalogs and Collections and child Items,
/// Catalogs and Collections.
///
/// A STAC Collection is represented in JSON format. Any JSON object that
/// contains all the required fields is a valid STAC Collection and also a valid
/// STAC Catalog.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Collection {
    #[serde(flatten)]
    core: CoreStruct,

    #[serde(skip_serializing_if = "Option::is_none")]
    title: Option<String>,

    description: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    keywords: Option<Vec<String>>,

    license: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    providers: Option<Vec<Provider>>,

    extent: Extent,

    #[serde(skip_serializing_if = "Option::is_none")]
    summaries: Option<Map<String, Value>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    assets: Option<HashMap<String, Asset>>,
}

impl Collection {
    /// Creates a new `Collection` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Collection, Core};
    /// let collection = Collection::new("an-id");
    /// assert_eq!(collection.id(), "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Collection {
        Collection {
            core: CoreStruct::new(COLLECTION_TYPE, id),
            title: None,
            description: String::new(),
            keywords: None,
            license: String::new(),
            providers: None,
            extent: Extent::default(),
            summaries: None,
            assets: None,
        }
    }

    /// Returns a reference to this Collection's title.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Collection;
    /// let collection = Collection::new("an-id");
    /// assert!(collection.title().is_none());
    /// ```
    pub fn title(&self) -> Option<&str> {
        self.title.as_deref()
    }

    /// Returns a reference to this Collection's description.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Collection;
    /// let collection = Collection::new("an-id");
    /// assert_eq!(collection.description(), "");
    /// ```
    pub fn description(&self) -> &str {
        &self.description
    }

    /// Returns a reference to this Collection's license.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Collection;
    /// let collection = Collection::new("an-id");
    /// assert_eq!(collection.license(), "");
    /// ```
    pub fn license(&self) -> &str {
        &self.license
    }

    /// Returns a reference to this Collection's providers.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Collection;
    /// let collection = Collection::new("an-id");
    /// assert!(collection.providers().is_none());
    /// ```
    pub fn providers(&self) -> Option<&[Provider]> {
        self.providers.as_deref()
    }

    /// Returns a reference to this Collection's extent.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Collection, Extent};
    /// let collection = Collection::new("an-id");
    /// assert_eq!(collection.extent(), &Extent::default());
    /// ```
    pub fn extent(&self) -> &Extent {
        &self.extent
    }

    /// Returns a reference to this Collection's summaries.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Collection;
    /// let collection = Collection::new("an-id");
    /// assert!(collection.summaries().is_none());
    /// ```
    pub fn summaries(&self) -> Option<&Map<String, Value>> {
        self.summaries.as_ref()
    }

    /// Returns a reference to this Collection's assets.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Collection;
    /// let collection = Collection::new("an-id");
    /// assert!(collection.assets().is_none());
    /// ```
    pub fn assets(&self) -> Option<&HashMap<String, Asset>> {
        self.assets.as_ref()
    }
}

impl AsRef<CoreStruct> for Collection {
    fn as_ref(&self) -> &CoreStruct {
        &self.core
    }
}

impl AsMut<CoreStruct> for Collection {
    fn as_mut(&mut self) -> &mut CoreStruct {
        &mut self.core
    }
}

impl Core for Collection {}

#[cfg(test)]
mod tests {
    use super::Collection;
    use crate::{Core, Extent, STAC_VERSION};

    #[test]
    fn new() {
        let collection = Collection::new("an-id");
        assert!(collection.title().is_none());
        assert_eq!(collection.description(), "");
        assert_eq!(collection.license(), "");
        assert!(collection.providers().is_none());
        assert_eq!(collection.extent(), &Extent::default());
        assert!(collection.summaries().is_none());
        assert!(collection.assets().is_none());
        assert_eq!(collection.type_(), "Collection");
        assert_eq!(collection.version(), STAC_VERSION);
        assert!(collection.extensions().is_none());
        assert_eq!(collection.id(), "an-id");
        assert!(collection.links().is_empty());
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
