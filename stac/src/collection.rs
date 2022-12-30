use crate::{Asset, Href, Link, Links, STAC_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// The type field for [Collections](Collection).
pub const COLLECTION_TYPE: &str = "Collection";

const DEFAULT_LICENSE: &str = "proprietary";

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

    #[serde(skip)]
    href: Option<String>,
}

/// This object provides information about a provider.
///
/// A provider is any of the organizations that captures or processes the
/// content of the [Collection](crate::Collection) and therefore influences the
/// data offered by this `Collection`. May also include information about the
/// final storage provider hosting the data.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Provider {
    /// The name of the organization or the individual.
    pub name: String,

    /// Multi-line description to add further provider information such as
    /// processing details for processors and producers, hosting details for
    /// hosts or basic contact information.
    ///
    /// [CommonMark 0.29](http://commonmark.org/) syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Roles of the provider.
    ///
    /// Any of `"licensor"`, `"producer"`, `"processor"`, or `"host"`.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,

    /// Homepage on which the provider describes the dataset and publishes contact information.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    /// Additional fields on the provider.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

/// The object describes the spatio-temporal extents of the [Collection](crate::Collection).
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Extent {
    /// Spatial extents covered by the `Collection`.
    pub spatial: SpatialExtent,
    /// Temporal extents covered by the `Collection`.
    pub temporal: TemporalExtent,

    /// Additional fields on the extent.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

/// The object describes the spatial extents of the Collection.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SpatialExtent {
    /// Potential spatial extents covered by the Collection.
    pub bbox: Vec<Vec<f64>>,
}

/// The object describes the temporal extents of the Collection.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TemporalExtent {
    /// Potential temporal extents covered by the Collection.
    pub interval: Vec<[Option<String>; 2]>,
}

impl Collection {
    /// Creates a new `Collection` with the given `id`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Collection;
    /// let collection = Collection::new("an-id", "a description");
    /// assert_eq!(collection.id, "an-id");
    /// assert_eq!(collection.description, "a description");
    /// ```
    pub fn new(id: impl ToString, description: impl ToString) -> Collection {
        Collection {
            r#type: COLLECTION_TYPE.to_string(),
            version: STAC_VERSION.to_string(),
            extensions: None,
            id: id.to_string(),
            title: None,
            description: description.to_string(),
            keywords: None,
            license: DEFAULT_LICENSE.to_string(),
            providers: None,
            extent: Extent::default(),
            summaries: None,
            links: Vec::new(),
            assets: None,
            additional_fields: Map::new(),
            href: None,
        }
    }
}

impl Href for Collection {
    fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    fn set_href(&mut self, href: impl ToString) {
        self.href = Some(href.to_string())
    }
}

impl Links for Collection {
    fn links(&self) -> &[Link] {
        &self.links
    }
    fn links_mut(&mut self) -> &mut Vec<Link> {
        &mut self.links
    }
}

impl Provider {
    /// Creates a new provider with the given name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Provider;
    /// let provider = Provider::new("a-name");
    /// assert_eq!(provider.name, "a-name");
    /// ```
    pub fn new(name: impl ToString) -> Provider {
        Provider {
            name: name.to_string(),
            description: None,
            roles: None,
            url: None,
            additional_fields: Map::new(),
        }
    }
}

impl Default for SpatialExtent {
    fn default() -> SpatialExtent {
        SpatialExtent {
            bbox: vec![vec![-180.0, -90.0, 180.0, 90.0]],
        }
    }
}

impl Default for TemporalExtent {
    fn default() -> TemporalExtent {
        TemporalExtent {
            interval: vec![[None, None]],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Collection, Extent, Provider};

    mod collection {
        use super::Collection;
        use crate::{Extent, STAC_VERSION};

        #[test]
        fn new() {
            let collection = Collection::new("an-id", "a description");
            assert!(collection.title.is_none());
            assert_eq!(collection.description, "a description");
            assert_eq!(collection.license, "proprietary");
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
            let collection = Collection::new("an-id", "a description");
            let value = serde_json::to_value(collection).unwrap();
            assert!(value.get("stac_extensions").is_none());
            assert!(value.get("title").is_none());
            assert!(value.get("keywords").is_none());
            assert!(value.get("providers").is_none());
            assert!(value.get("summaries").is_none());
            assert!(value.get("assets").is_none());
        }
    }

    mod provider {
        use super::Provider;

        #[test]
        fn new() {
            let provider = Provider::new("a-name");
            assert_eq!(provider.name, "a-name");
            assert!(provider.description.is_none());
            assert!(provider.roles.is_none());
            assert!(provider.url.is_none());
            assert!(provider.additional_fields.is_empty());
        }

        #[test]
        fn skip_serializing() {
            let provider = Provider::new("an-id");
            let value = serde_json::to_value(provider).unwrap();
            assert!(value.get("description").is_none());
            assert!(value.get("roles").is_none());
            assert!(value.get("url").is_none());
        }
    }

    mod extent {
        use super::Extent;

        #[test]
        fn default() {
            let extent = Extent::default();
            assert_eq!(extent.spatial.bbox, [[-180.0, -90.0, 180.0, 90.0]]);
            assert_eq!(extent.temporal.interval, [[None, None]]);
            assert!(extent.additional_fields.is_empty());
        }
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
