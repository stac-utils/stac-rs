use crate::{Href, Link, STAC_VERSION};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// A trait derived by all three core STAC object types.
///
/// This trait provides access to the field shared by all three STAC object types.
pub trait Core: AsRef<CoreStruct> + AsMut<CoreStruct> {
    /// Returns a reference to this structure's type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Core, Catalog};
    /// let catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.type_(), "Catalog");
    /// ```
    fn type_(&self) -> &str {
        &self.as_ref().type_
    }

    /// Returns a reference to this structure's STAC version.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Core, Catalog};
    /// let catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.version(), stac::STAC_VERSION);
    /// ```
    fn version(&self) -> &str {
        &self.as_ref().version
    }

    /// Returns a reference to this structure's extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Core, Catalog};
    /// let catalog = Catalog::new("an-id");
    /// assert!(catalog.extensions().is_none());
    /// ```
    fn extensions(&self) -> Option<&[String]> {
        self.as_ref().extensions.as_deref()
    }

    /// Returns a reference to this structure's id.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Core, Catalog};
    /// let catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.id(), "an-id");
    /// ```
    fn id(&self) -> &str {
        &self.as_ref().id
    }

    /// Sets this object's id.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Core, Catalog};
    /// let mut catalog = Catalog::new("an-id");
    /// catalog.set_id("a-new-id");
    /// assert_eq!(catalog.id(), "a-new-id");
    fn set_id<T: ToString>(&mut self, id: T) {
        self.as_mut().id = id.to_string();
    }

    /// Returns a reference to this structure's links.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Core, Catalog};
    /// let catalog = Catalog::new("an-id");
    /// assert!(catalog.links().is_empty());
    /// ```
    fn links(&self) -> &[Link] {
        &self.as_ref().links
    }

    /// Returns a reference to this structure's additional fields.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Core, Catalog};
    /// let catalog = Catalog::new("an-id");
    /// assert!(catalog.additional_fields().is_empty());
    /// ```
    fn additional_fields(&self) -> &Map<String, Value> {
        &self.as_ref().additional_fields
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct CoreStruct {
    #[serde(rename = "type")]
    type_: String,

    #[serde(rename = "stac_version")]
    version: String,

    #[serde(rename = "stac_extensions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    extensions: Option<Vec<String>>,

    id: String,

    links: Vec<Link>,

    #[serde(flatten)]
    additional_fields: Map<String, Value>,

    #[serde(skip)]
    pub(crate) href: Option<Href>,
}

impl CoreStruct {
    pub(crate) fn new<A: ToString, B: ToString>(type_: A, id: B) -> CoreStruct {
        CoreStruct {
            type_: type_.to_string(),
            version: STAC_VERSION.to_string(),
            extensions: None,
            id: id.to_string(),
            links: Vec::new(),
            additional_fields: Map::new(),
            href: None,
        }
    }
}
