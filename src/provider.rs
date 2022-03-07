use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
