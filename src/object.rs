use crate::{
    catalog::{Catalog, CATALOG_TYPE},
    collection::{Collection, COLLECTION_TYPE},
    item::{Item, ITEM_TYPE},
    Error,
};
use serde_json::Value;
use std::convert::TryFrom;

/// A STAC object.
///
/// Can be an Item, Catalog, or Collection.
#[derive(Debug, Clone)]
pub enum Object {
    /// A STAC Item.
    Item(Item),
    /// A STAC Catalog.
    Catalog(Catalog),
    /// A STAC Collection.
    Collection(Collection),
}

impl Object {
    /// Returns true if this object is an Item.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Catalog, Item};
    /// let object = Object::Item(Item::new("an-id"));
    /// assert!(object.is_item());
    /// let object = Object::Catalog(Catalog::new("an-id"));
    /// assert!(!object.is_item());
    /// ```
    pub fn is_item(&self) -> bool {
        matches!(self, Object::Item(_))
    }

    /// Returns a reference to the underlying Item.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Item};
    /// let object = Object::Item(Item::new("an-id"));
    /// let item = object.as_item().unwrap();
    /// ```
    pub fn as_item(&self) -> Option<&Item> {
        match self {
            Object::Item(item) => Some(item),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying Item.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Item};
    /// let mut object = Object::Item(Item::new("an-id"));
    /// let item = object.as_item_mut().unwrap();
    /// ```
    pub fn as_item_mut(&mut self) -> Option<&mut Item> {
        match self {
            Object::Item(item) => Some(item),
            _ => None,
        }
    }

    /// Returns true if this object is a Catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Catalog, Item};
    /// let object = Object::Catalog(Catalog::new("an-id"));
    /// assert!(object.is_catalog());
    /// let object = Object::Item(Item::new("an-id"));
    /// assert!(!object.is_catalog());
    /// ```
    pub fn is_catalog(&self) -> bool {
        matches!(self, Object::Catalog(_))
    }

    /// Returns a reference to the underlying Catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Catalog};
    /// let object = Object::Catalog(Catalog::new("an-id"));
    /// let catalog = object.as_catalog().unwrap();
    /// ```
    pub fn as_catalog(&self) -> Option<&Catalog> {
        match self {
            Object::Catalog(catalog) => Some(catalog),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying Catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Catalog};
    /// let mut object = Object::Catalog(Catalog::new("an-id"));
    /// let catalog = object.as_catalog_mut().unwrap();
    /// ```
    pub fn as_catalog_mut(&mut self) -> Option<&mut Catalog> {
        match self {
            Object::Catalog(catalog) => Some(catalog),
            _ => None,
        }
    }

    /// Returns true if this object is a Collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Collection, Item};
    /// let object = Object::Collection(Collection::new("an-id"));
    /// assert!(object.is_collection());
    /// let object = Object::Item(Item::new("an-id"));
    /// assert!(!object.is_collection());
    /// ```
    pub fn is_collection(&self) -> bool {
        matches!(self, Object::Collection(_))
    }

    /// Converts this object to a serde_json::Value, if possible.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Item};
    /// let object = Object::Item(Item::new("an-id"));
    /// let value = object.to_value().unwrap();
    /// ```
    pub fn to_value(&self) -> Result<Value, Error> {
        match self {
            Object::Item(item) => serde_json::to_value(item).map_err(Error::from),
            Object::Catalog(catalog) => serde_json::to_value(catalog).map_err(Error::from),
            Object::Collection(collection) => serde_json::to_value(collection).map_err(Error::from),
        }
    }

    /// Returns a reference to the underlying Collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Collection};
    /// let object = Object::Collection(Collection::new("an-id"));
    /// let collection = object.as_collection().unwrap();
    /// ```
    pub fn as_collection(&self) -> Option<&Collection> {
        match self {
            Object::Collection(collection) => Some(collection),
            _ => None,
        }
    }

    /// Returns a mutable reference to the underlying Collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Collection};
    /// let mut object = Object::Collection(Collection::new("an-id"));
    /// let collection = object.as_collection_mut().unwrap();
    /// ```
    pub fn as_collection_mut(&mut self) -> Option<&mut Collection> {
        match self {
            Object::Collection(collection) => Some(collection),
            _ => None,
        }
    }
}

impl From<Item> for Object {
    fn from(item: Item) -> Object {
        Object::Item(item)
    }
}

impl From<Catalog> for Object {
    fn from(catalog: Catalog) -> Object {
        Object::Catalog(catalog)
    }
}

impl From<Collection> for Object {
    fn from(collection: Collection) -> Object {
        Object::Collection(collection)
    }
}

impl TryFrom<Value> for Object {
    type Error = Error;
    fn try_from(value: Value) -> Result<Object, Error> {
        match value.get("type").and_then(|v| v.as_str()) {
            Some(ITEM_TYPE) => Ok(Object::Item(serde_json::from_value(value)?)),
            Some(CATALOG_TYPE) => Ok(Object::Catalog(serde_json::from_value(value)?)),
            Some(COLLECTION_TYPE) => Ok(Object::Collection(serde_json::from_value(value)?)),
            Some(v) => Err(Error::InvalidTypeValue(v.to_string())),
            None => Err(Error::MissingType),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Object;
    use serde_json::Value;
    use std::{convert::TryFrom, fs::File, io::BufReader};

    #[test]
    fn unknown_type() {
        let file = File::open("data/simple-item.json").unwrap();
        let buf_reader = BufReader::new(file);
        let mut value: Value = serde_json::from_reader(buf_reader).unwrap();
        let type_ = value.get_mut("type").unwrap();
        *type_ = Value::String("not-a-type".to_string());
        assert!(Object::try_from(value).is_err());
    }

    #[test]
    fn missing_type() {
        let file = File::open("data/simple-item.json").unwrap();
        let buf_reader = BufReader::new(file);
        let mut value: Value = serde_json::from_reader(buf_reader).unwrap();
        let object = value.as_object_mut().unwrap();
        let _ = object.remove("type");
        assert!(Object::try_from(value).is_err());
    }
}
