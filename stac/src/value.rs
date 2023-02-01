use crate::{
    Catalog, Collection, Error, Href, Item, ItemCollection, Link, Links, Result, CATALOG_TYPE,
    COLLECTION_TYPE, ITEM_COLLECTION_TYPE, ITEM_TYPE,
};
use serde_json::Map;
use std::convert::TryFrom;

/// An enum that can hold any STAC object type.
#[derive(Clone, Debug)]
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
    /// Converts a [serde_json::Value] to a Value.
    ///
    /// Uses the `type` field to determine object type.
    ///
    /// # Examples
    ///
    /// ```
    /// use serde_json::json;
    /// use stac::Value;
    /// let catalog = json!({
    ///     "type": "Catalog",
    ///     "stac_version": "1.0.0",
    ///     "id": "an-id",
    ///     "description": "a description",
    ///     "links": []
    /// });
    /// let catalog = Value::from_json(catalog).unwrap();
    /// ```
    pub fn from_json(value: serde_json::Value) -> Result<Value> {
        if let Some(r#type) = value.get("type") {
            if let Some(r#type) = r#type.as_str() {
                match r#type {
                    CATALOG_TYPE => serde_json::from_value::<Catalog>(value)
                        .map(Value::Catalog)
                        .map_err(Error::from),
                    COLLECTION_TYPE => serde_json::from_value::<Collection>(value)
                        .map(Value::Collection)
                        .map_err(Error::from),
                    ITEM_TYPE => serde_json::from_value::<Item>(value)
                        .map(Value::Item)
                        .map_err(Error::from),
                    ITEM_COLLECTION_TYPE => serde_json::from_value::<ItemCollection>(value)
                        .map(Value::ItemCollection)
                        .map_err(Error::from),
                    _ => Err(Error::UnknownType(r#type.to_string())),
                }
            } else {
                Err(Error::InvalidTypeField(r#type.clone()))
            }
        } else {
            Err(Error::MissingType)
        }
    }

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
    /// let value = stac::read("data/simple-item.json").unwrap();
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

impl Href for Value {
    fn href(&self) -> Option<&str> {
        use Value::*;
        match self {
            Catalog(catalog) => catalog.href(),
            Collection(collection) => collection.href(),
            Item(item) => item.href(),
            ItemCollection(item_collection) => item_collection.href(),
        }
    }

    fn set_href(&mut self, href: impl ToString) {
        use Value::*;
        match self {
            Catalog(catalog) => catalog.set_href(href),
            Collection(collection) => collection.set_href(href),
            Item(item) => item.set_href(href),
            ItemCollection(item_collection) => item_collection.set_href(href),
        }
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

impl TryFrom<Value> for serde_json::Value {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        use Value::*;
        match value {
            Catalog(catalog) => {
                if catalog.r#type == CATALOG_TYPE {
                    serde_json::to_value(catalog).map_err(Error::from)
                } else {
                    Err(Error::IncorrectType {
                        actual: catalog.r#type,
                        expected: CATALOG_TYPE.to_string(),
                    })
                }
            }
            Collection(collection) => {
                if collection.r#type == COLLECTION_TYPE {
                    serde_json::to_value(collection).map_err(Error::from)
                } else {
                    Err(Error::IncorrectType {
                        actual: collection.r#type,
                        expected: COLLECTION_TYPE.to_string(),
                    })
                }
            }
            Item(item) => {
                if item.r#type == ITEM_TYPE {
                    serde_json::to_value(item).map_err(Error::from)
                } else {
                    Err(Error::IncorrectType {
                        actual: item.r#type,
                        expected: ITEM_TYPE.to_string(),
                    })
                }
            }
            ItemCollection(item_collection) => {
                if item_collection.r#type == ITEM_COLLECTION_TYPE {
                    serde_json::to_value(item_collection).map_err(Error::from)
                } else {
                    Err(Error::IncorrectType {
                        actual: item_collection.r#type,
                        expected: ITEM_COLLECTION_TYPE.to_string(),
                    })
                }
            }
        }
    }
}

impl TryFrom<Value> for Map<String, serde_json::Value> {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        if let serde_json::Value::Object(object) = serde_json::Value::try_from(value)? {
            Ok(object)
        } else {
            panic!("all STAC values should serialize to a serde_json::Value::Object")
        }
    }
}

impl TryFrom<Value> for Item {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        if let Value::Item(item) = value {
            Ok(item)
        } else {
            Err(Error::NotAnItem(value))
        }
    }
}

impl TryFrom<Value> for Catalog {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        if let Value::Catalog(catalog) = value {
            Ok(catalog)
        } else {
            Err(Error::NotACatalog(value))
        }
    }
}

impl TryFrom<Value> for Collection {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        if let Value::Collection(collection) = value {
            Ok(collection)
        } else {
            Err(Error::NotACollection(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Value;
    use crate::{Catalog, Collection, Error, Item};
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
        let catalog = Value::from_json(catalog).unwrap();
        assert!(catalog.is_catalog());
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
        let collection = Value::from_json(collection).unwrap();
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
        let item = Value::from_json(item).unwrap();
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
        assert!(matches!(
            Value::from_json(catalog).unwrap_err(),
            Error::UnknownType(_)
        ))
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
        assert!(matches!(
            Value::from_json(catalog).unwrap_err(),
            Error::InvalidTypeField(_)
        ))
    }

    #[test]
    fn from_json_missing_type_field() {
        let catalog = json!({
            "stac_version": "1.0.0",
            "id": "an-id",
            "description": "a description",
            "links": []
        });
        assert!(matches!(
            Value::from_json(catalog).unwrap_err(),
            Error::MissingType
        ))
    }

    #[test]
    fn catalog_into_json_incorrect_type() {
        let mut catalog = Catalog::new("an-id", "a description");
        catalog.r#type = "Schmatalog".to_string();
        assert!(matches!(
            serde_json::Value::try_from(Value::Catalog(catalog)).unwrap_err(),
            Error::IncorrectType { .. }
        ))
    }

    #[test]
    fn collection_into_json_incorrect_type() {
        let mut collection = Collection::new("an-id", "a description");
        collection.r#type = "Scmalection".to_string();
        assert!(matches!(
            serde_json::Value::try_from(Value::Collection(collection)).unwrap_err(),
            Error::IncorrectType { .. }
        ))
    }

    #[test]
    fn item_into_json_incorrect_type() {
        let mut item = Item::new("an-id");
        item.r#type = "Item".to_string();
        assert!(matches!(
            serde_json::Value::try_from(Value::Item(item)).unwrap_err(),
            Error::IncorrectType { .. }
        ))
    }
}
