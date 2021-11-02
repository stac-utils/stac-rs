use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct Asset {
    pub href: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub type_: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
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
    pub fn new<S: ToString>(href: S) -> Asset {
        Asset {
            href: href.to_string(),
            title: None,
            description: None,
            type_: None,
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
        assert!(asset.type_.is_none());
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
