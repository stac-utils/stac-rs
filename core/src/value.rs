use crate::{
    Catalog, Collection, Error, Item, ItemCollection, Link, Links, Migrate, Object, Result, Version,
};
use serde::{Deserialize, Serialize};
use serde_json::Map;
use std::convert::TryFrom;

/// An enum that can hold any STAC object type.
#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
#[serde(untagged)]
pub enum Value {
    /// A STAC Item.
    Item(Item),

    /// A STAC Catalog.
    Catalog(Catalog),

    /// A STAC Collection.
    Collection(Collection),

    /// An ItemCollection.
    ItemCollection(ItemCollection),
}

impl Value {
    /// Returns true if this is a catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Catalog};
    /// assert!(Value::Catalog(Catalog::new("an-id", "a description")).is_catalog());
    /// ```
    pub fn is_catalog(&self) -> bool {
        matches!(self, Value::Catalog(_))
    }

    /// Returns a reference to this value as a catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Catalog};
    /// let value = Value::Catalog(Catalog::new("an-id", "a description"));
    /// assert_eq!(value.as_catalog().unwrap().id, "an-id");
    /// ```
    pub fn as_catalog(&self) -> Option<&Catalog> {
        if let Value::Catalog(catalog) = self {
            Some(catalog)
        } else {
            None
        }
    }

    /// Returns a mutable reference to this value as a catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Catalog};
    /// let mut value = Value::Catalog(Catalog::new("an-id", "a description"));
    /// value.as_mut_catalog().unwrap().id = "another-id".to_string();
    /// ```
    pub fn as_mut_catalog(&mut self) -> Option<&mut Catalog> {
        if let Value::Catalog(catalog) = self {
            Some(catalog)
        } else {
            None
        }
    }

    /// Returns true if this is a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Collection};
    /// assert!(Value::Collection(Collection::new("an-id", "a description")).is_collection());
    /// ```
    pub fn is_collection(&self) -> bool {
        matches!(self, Value::Collection(_))
    }

    /// Returns a reference to this value as a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Collection};
    /// let value = Value::Collection(Collection::new("an-id", "a description"));
    /// assert_eq!(value.as_collection().unwrap().id, "an-id");
    /// ```
    pub fn as_collection(&self) -> Option<&Collection> {
        if let Value::Collection(collection) = self {
            Some(collection)
        } else {
            None
        }
    }

    /// Returns a mutable reference to this value as a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Collection};
    /// let mut value = Value::Collection(Collection::new("an-id", "a description"));
    /// value.as_mut_collection().unwrap().id = "another-id".to_string();
    /// ```
    pub fn as_mut_collection(&mut self) -> Option<&mut Collection> {
        if let Value::Collection(collection) = self {
            Some(collection)
        } else {
            None
        }
    }

    /// Returns true if this is an item.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Item};
    /// assert!(Value::Item(Item::new("an-id")).is_item());
    /// ```
    pub fn is_item(&self) -> bool {
        matches!(self, Value::Item(_))
    }

    /// Returns a reference to this value as an item.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Item};
    /// let value = Value::Item(Item::new("an-id"));
    /// assert_eq!(value.as_item().unwrap().id, "an-id");
    /// ```
    pub fn as_item(&self) -> Option<&Item> {
        if let Value::Item(item) = self {
            Some(item)
        } else {
            None
        }
    }

    /// Returns a mutable reference to this value as an item.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Item};
    /// let mut value = Value::Item(Item::new("an-id"));
    /// value.as_mut_item().unwrap().id = "another-id".to_string();
    /// ```
    pub fn as_mut_item(&mut self) -> Option<&mut Item> {
        if let Value::Item(item) = self {
            Some(item)
        } else {
            None
        }
    }

    /// Returns this value's type name.
    ///
    /// This is "Item", "Catalog", "Collection", or "ItemCollection".
    ///
    /// We can't just use the `type` field, because items' type field is "Feature".
    ///
    /// # Examples
    ///
    /// ```
    /// let value: stac::Value = stac::read("examples/simple-item.json").unwrap();
    /// assert_eq!(value.type_name(), "Item");
    /// ```
    pub fn type_name(&self) -> &'static str {
        use Value::*;
        match self {
            Item(_) => "Item",
            Collection(_) => "Collection",
            Catalog(_) => "Catalog",
            ItemCollection(_) => "ItemCollection",
        }
    }
}

impl Object for Value {
    const TYPE: &str = "Value";

    fn href(&self) -> Option<&str> {
        use Value::*;
        match self {
            Catalog(catalog) => catalog.href(),
            Collection(collection) => collection.href(),
            Item(item) => item.href(),
            ItemCollection(item_collection) => item_collection.href(),
        }
    }
    fn href_mut(&mut self) -> &mut Option<String> {
        use Value::*;
        match self {
            Catalog(catalog) => catalog.href_mut(),
            Collection(collection) => collection.href_mut(),
            Item(item) => item.href_mut(),
            ItemCollection(item_collection) => item_collection.href_mut(),
        }
    }

    #[cfg(feature = "geoparquet")]
    fn geoparquet_from_bytes(bytes: impl Into<bytes::Bytes>) -> Result<Self> {
        ItemCollection::geoparquet_from_bytes(bytes).map(Self::from)
    }
}

impl Links for Value {
    fn links(&self) -> &[Link] {
        use Value::*;
        match self {
            Catalog(catalog) => catalog.links(),
            Collection(collection) => collection.links(),
            Item(item) => item.links(),
            ItemCollection(item_collection) => item_collection.links(),
        }
    }

    fn links_mut(&mut self) -> &mut Vec<Link> {
        use Value::*;
        match self {
            Catalog(catalog) => catalog.links_mut(),
            Collection(collection) => collection.links_mut(),
            Item(item) => item.links_mut(),
            ItemCollection(item_collection) => item_collection.links_mut(),
        }
    }
}

impl TryFrom<Value> for Map<String, serde_json::Value> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        if let serde_json::Value::Object(object) = serde_json::to_value(value)? {
            Ok(object)
        } else {
            panic!("all STAC values should serialize to a serde_json::Value::Object")
        }
    }
}

impl From<Item> for Value {
    fn from(item: Item) -> Self {
        Value::Item(item)
    }
}

impl From<Catalog> for Value {
    fn from(catalog: Catalog) -> Self {
        Value::Catalog(catalog)
    }
}

impl From<Collection> for Value {
    fn from(collection: Collection) -> Self {
        Value::Collection(collection)
    }
}

impl From<ItemCollection> for Value {
    fn from(item_collection: ItemCollection) -> Self {
        Value::ItemCollection(item_collection)
    }
}

impl Migrate for Value {
    fn migrate(self, version: &Version) -> Result<Value> {
        match self {
            Value::Item(item) => item.migrate(version).map(Value::Item),
            Value::Catalog(catalog) => catalog.migrate(version).map(Value::Catalog),
            Value::Collection(collection) => collection.migrate(version).map(Value::Collection),
            Value::ItemCollection(item_collection) => {
                item_collection.migrate(version).map(Value::ItemCollection)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Value;
    use serde_json::json;

    #[test]
    fn catalog_from_json() {
        let catalog = json!({
            "type": "Catalog",
            "stac_version": "1.0.0",
            "id": "an-id",
            "description": "a description",
            "links": []
        });
        let value: Value = serde_json::from_value(catalog).unwrap();
        assert!(value.is_catalog());
    }

    #[test]
    fn collection_from_json() {
        let collection = json!({
            "type": "Collection",
            "stac_version": "1.0.0",
            "id": "an-id",
            "description": "a description",
            "license": "proprietary",
            "extent": {
                "spatial": [[]],
                "temporal": [[]]
            },
            "links": []
        });
        let collection: Value = serde_json::from_value(collection).unwrap();
        assert!(collection.is_collection());
    }

    #[test]
    fn item_from_json() {
        let item = json!({
            "type": "Feature",
            "stac_version": "1.0.0",
            "id": "an-id",
            "geometry": null,
            "properties": {},
            "links": [],
            "assets": {}
        });
        let item: Value = serde_json::from_value(item).unwrap();
        assert!(item.is_item());
    }

    #[test]
    fn from_json_unknown_type() {
        let catalog = json!({
            "type": "Schmatalog",
            "stac_version": "1.0.0",
            "id": "an-id",
            "description": "a description",
            "links": []
        });
        assert!(serde_json::from_value::<Value>(catalog).is_err());
    }

    #[test]
    fn from_json_invalid_type_field() {
        let catalog = json!({
            "type": {"foo": "bar"},
            "stac_version": "1.0.0",
            "id": "an-id",
            "description": "a description",
            "links": []
        });
        assert!(serde_json::from_value::<Value>(catalog).is_err());
    }

    #[test]
    fn from_json_missing_type_field() {
        let catalog = json!({
            "stac_version": "1.0.0",
            "id": "an-id",
            "description": "a description",
            "links": []
        });
        assert!(serde_json::from_value::<Value>(catalog).is_err());
    }
}
