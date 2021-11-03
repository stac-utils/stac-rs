use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// This object describes a relationship with another entity.
///
/// Data providers are advised to be liberal with the links section, to describe
/// things like the Catalog an Item is in, related Items, parent or child Items
/// (modeled in different ways, like an 'acquisition' or derived data). It is
/// allowed to add additional fields such as a title and type.
#[derive(Debug, Serialize, Deserialize)]
pub struct Link {
    /// The actual link in the format of an URL.
    ///
    /// Relative and absolute links are both allowed.
    pub href: String,

    /// Relationship between the current document and the linked document.
    ///
    /// See chapter "Relation types" for more information.
    pub rel: String,

    /// Media type of the referenced entity.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,

    /// A human readable title to be used in rendered displays of the link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// Additional fields on the link.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl Link {
    /// Creates a new link with the provided href and rel type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "a-rel");
    /// assert_eq!(link.href, "an-href");
    /// assert_eq!(link.rel, "a-rel");
    /// ```
    pub fn new<S0: ToString, S1: ToString>(href: S0, rel: S1) -> Link {
        Link {
            href: href.to_string(),
            rel: rel.to_string(),
            type_: None,
            title: None,
            additional_fields: Map::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Link;

    #[test]
    fn new() {
        let link = Link::new("an-href", "a-rel");
        assert_eq!(link.href, "an-href");
        assert_eq!(link.rel, "a-rel");
        assert!(link.type_.is_none());
        assert!(link.title.is_none());
    }

    #[test]
    fn skip_serializing() {
        let link = Link::new("an-href", "a-rel");
        let value = serde_json::to_value(link).unwrap();
        assert!(value.get("type").is_none());
        assert!(value.get("title").is_none());
    }
}
