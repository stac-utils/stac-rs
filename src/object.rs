use crate::{
    Catalog, Collection, Error, Href, Item, Link, CATALOG_TYPE, COLLECTION_TYPE, ITEM_TYPE,
};
use serde_json::Value;

const TYPE_FIELD: &str = "type";

/// A STAC object.
///
/// Holds both the inner STAC object structure, e.g. an [Item], and an optional
/// [Href] to where the object "lives".
#[derive(Debug)]
pub struct Object {
    /// The href where this object "lives".
    pub href: Option<Href>,
    inner: Inner,
}

#[derive(Debug)]
enum Inner {
    Item(Item),
    Catalog(Catalog),
    Collection(Collection),
}

impl Object {
    /// Returns a reference to this object as a [Catalog], or None if it is not a catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// let catalog = stac::read("data/catalog.json").unwrap();
    /// println!("Description: {}", catalog.as_catalog().unwrap().description);
    /// ```
    pub fn as_catalog(&self) -> Option<&Catalog> {
        match &self.inner {
            Inner::Catalog(catalog) => Some(catalog),
            _ => None,
        }
    }

    /// Returns a mutable reference to this object as a [Catalog], or None if it is not a Catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// let catalog = stac::read("data/catalog.json").unwrap();
    /// println!("Description: {}", catalog.as_catalog().unwrap().description);
    /// ```
    pub fn as_mut_catalog(&mut self) -> Option<&mut Catalog> {
        match &mut self.inner {
            Inner::Catalog(catalog) => Some(catalog),
            _ => None,
        }
    }

    // TODO add as_* and as_mut_* methods

    /// Returns true if this object is an [Item].
    ///
    /// # Examples
    ///
    /// ```
    /// let item = stac::read("data/simple-item.json").unwrap();
    /// assert!(item.is_item());
    /// ```
    pub fn is_item(&self) -> bool {
        matches!(self.inner, Inner::Item(_))
    }

    /// Returns this object's inner [Item], or `None` if it is not an item.
    ///
    /// # Examples
    ///
    /// ```
    /// let item = stac::read("data/simple-item.json").unwrap();
    /// let item = item.into_item().unwrap();
    /// ```
    pub fn into_item(self) -> Option<Item> {
        match self.inner {
            Inner::Item(item) => Some(item),
            _ => None,
        }
    }

    /// Create a STAC Object from a JSON value.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Object;
    /// let file = std::fs::File::open("data/catalog.json").unwrap();
    /// let reader = std::io::BufReader::new(file);
    /// let value: serde_json::Value = serde_json::from_reader(reader).unwrap();
    /// let object = Object::from_value(value).unwrap();
    /// ```
    pub fn from_value(value: Value) -> Result<Object, Error> {
        if let Some(type_) = value.get(TYPE_FIELD) {
            if let Some(type_) = type_.as_str() {
                match type_ {
                    ITEM_TYPE => Ok(Object {
                        inner: Inner::Item(serde_json::from_value(value)?),
                        href: None,
                    }),
                    CATALOG_TYPE => Ok(Object {
                        inner: Inner::Catalog(serde_json::from_value(value)?),
                        href: None,
                    }),
                    COLLECTION_TYPE => Ok(Object {
                        inner: Inner::Collection(serde_json::from_value(value)?),
                        href: None,
                    }),
                    _ => Err(Error::InvalidTypeValue(type_.to_string())),
                }
            } else {
                Err(Error::InvalidTypeField(type_.clone()))
            }
        } else {
            Err(Error::MissingType)
        }
    }

    /// Returns a reference to this object's id.
    ///
    /// # Examples
    ///
    /// ```
    /// let object = stac::read("data/catalog.json").unwrap();
    /// assert_eq!(object.id(), "examples");
    /// ```
    pub fn id(&self) -> &str {
        match &self.inner {
            Inner::Item(item) => &item.id,
            Inner::Catalog(catalog) => &catalog.id,
            Inner::Collection(collection) => &collection.id,
        }
    }

    /// Returns a reference to this object's links.
    ///
    /// # Examples
    ///
    /// ```
    /// let object = stac::read("data/catalog.json").unwrap();
    /// let links = object.links();
    /// ```
    pub fn links(&self) -> &[Link] {
        match &self.inner {
            Inner::Item(item) => &item.links,
            Inner::Catalog(catalog) => &catalog.links,
            Inner::Collection(collection) => &collection.links,
        }
    }
}
