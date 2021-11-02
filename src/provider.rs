use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[derive(Debug, Serialize, Deserialize)]
pub struct Provider {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub roles: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
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
    pub fn new<S: ToString>(name: S) -> Provider {
        Provider {
            name: name.to_string(),
            description: None,
            roles: None,
            url: None,
            additional_fields: Map::new(),
        }
    }
}

#[cfg(test)]
mod tests {
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
