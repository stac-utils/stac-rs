use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

/// An Asset is an object that contains a URI to data associated with the [Item](crate::Item) that can be downloaded or streamed.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Asset {
    /// URI to the asset object.
    ///
    /// Relative and absolute URIs are both allowed.
    pub href: String,

    /// The displayed title for clients and users.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// A description of the Asset providing additional details, such as how it was processed or created.
    ///
    /// [CommonMark 0.29](http://commonmark.org/) syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// [Media type](crate::media_type) of the asset.
    ///
    /// See the [common media types](https://github.com/radiantearth/stac-spec/blob/master/best-practices.md#common-media-types-in-stac) in the best practice doc for commonly used asset types.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// The semantic roles of the asset, similar to the use of rel in [Links](crate::Link).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,

    /// Additional fields on the asset.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

/// Trait implemented by anything that has assets.
///
/// As of STAC v1.0.0, this is [Collection](crate::Collection) and [Item](crate::Item).
pub trait Assets {
    /// Returns a reference to this object's assets.
    ///
    /// # Examples
    ///
    /// [Item](crate::Item) has assets:
    ///
    /// ```
    /// use stac::{Item, Assets};
    /// let item: Item = stac::read("data/simple-item.json").unwrap();
    /// assert!(!item.assets().is_empty());
    /// ```
    fn assets(&self) -> &HashMap<String, Asset>;

    /// Returns a mut reference to this object's assets.
    ///
    /// # Examples
    ///
    /// [Item](crate::Item) has assets:
    ///
    /// ```
    /// use stac::{Item, Asset, Assets};
    /// let mut item: Item = stac::read("data/simple-item.json").unwrap();
    /// item.assets_mut().insert("foo".to_string(), Asset::new("./asset.tif"));
    /// ```
    fn assets_mut(&mut self) -> &mut HashMap<String, Asset>;
}

impl Asset {
    /// Creates a new asset with the provided href.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Asset;
    /// let asset = Asset::new("an-href");
    /// assert_eq!(asset.href, "an-href");
    /// ```
    pub fn new(href: impl ToString) -> Asset {
        Asset {
            href: href.to_string(),
            title: None,
            description: None,
            r#type: None,
            roles: None,
            additional_fields: Map::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Asset;

    #[test]
    fn new() {
        let asset = Asset::new("an-href");
        assert_eq!(asset.href, "an-href");
        assert!(asset.title.is_none());
        assert!(asset.description.is_none());
        assert!(asset.r#type.is_none());
        assert!(asset.roles.is_none());
    }

    #[test]
    fn skip_serializing() {
        let asset = Asset::new("an-href");
        let value = serde_json::to_value(asset).unwrap();
        assert!(value.get("title").is_none());
        assert!(value.get("description").is_none());
        assert!(value.get("type").is_none());
        assert!(value.get("roles").is_none());
    }
}
