use crate::{Asset, Link, STAC_VERSION};
use chrono::Utc;
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

const ITEM_TYPE: &str = "Feature";

#[derive(Debug, Serialize, Deserialize)]
pub struct Item {
    #[serde(rename = "type")]
    pub type_: String,
    #[serde(rename = "stac_version")]
    pub version: String,
    #[serde(rename = "stac_extensions")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub extensions: Option<Vec<String>>,
    pub id: String,
    pub geometry: Option<Geometry>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<f64>>,
    pub properties: Properties,
    pub links: Vec<Link>,
    pub assets: HashMap<String, Asset>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collection: Option<String>,
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Properties {
    pub datetime: Option<String>,
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl Item {
    /// Creates a new `Item` with the given `id`.
    ///
    /// The item properties' `datetime` field is set to the creation time.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Item;
    /// let item = Item::new("an-id");
    /// assert_eq!(item.id, "an-id");
    /// ```
    pub fn new<S: ToString>(id: S) -> Item {
        Item {
            type_: ITEM_TYPE.to_string(),
            version: STAC_VERSION.to_string(),
            extensions: None,
            id: id.to_string(),
            geometry: None,
            bbox: None,
            properties: Properties {
                datetime: Some(Utc::now().to_rfc3339()),
                additional_fields: Map::new(),
            },
            links: Vec::new(),
            assets: HashMap::new(),
            collection: None,
            additional_fields: Map::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Item;
    use crate::STAC_VERSION;

    #[test]
    fn new() {
        let item = Item::new("an-id");
        assert_eq!(item.type_, "Feature");
        assert_eq!(item.version, STAC_VERSION);
        assert!(item.extensions.is_none());
        assert_eq!(item.id, "an-id");
        assert_eq!(item.geometry, None);
        assert!(item.properties.datetime.is_some());
        assert!(item.links.is_empty());
        assert!(item.assets.is_empty());
        assert!(item.collection.is_none());
    }

    #[test]
    fn skip_serializing() {
        let item = Item::new("an-id");
        let value = serde_json::to_value(item).unwrap();
        assert!(value.get("stac_extensions").is_none());
        assert!(value.get("bbox").is_none());
        assert!(value.get("collection").is_none());
    }
    mod roundtrip {
        use super::Item;
        use crate::tests::roundtrip;

        roundtrip!(simple_item, "examples/simple-item.json", Item);
        roundtrip!(extended_item, "examples/extended-item.json", Item);
        roundtrip!(core_item, "examples/core-item.json", Item);
        roundtrip!(
            collectionless_item,
            "examples/collectionless-item.json",
            Item
        );
        roundtrip!(
            proj_example_item,
            "examples/extensions-collection/proj-example/proj-example.json",
            Item
        );
    }
}
