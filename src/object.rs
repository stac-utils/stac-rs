use crate::{
    Catalog, Collection, Error, Href, Item, Link, CATALOG_TYPE, COLLECTION_TYPE, ITEM_TYPE,
};

const TYPE_FIELD: &str = "type";

/// A wrapper around any of the three main STAC entities: [Item], [Catalog], and [Collection].
///
/// Holds both the inner STAC object structure, e.g. an [Item], and an optional
/// [Href] to where the object was read from or should be written to.  Objects
/// can be created by reading JSON, because the actual type of the STAC object
/// cannot be known before reading:
///
/// ```
/// let object = stac::read("data/catalog.json").unwrap();
/// ```
#[derive(Debug, PartialEq, Clone)]
pub struct Object {
    /// An href to where the object was read from or will be written to.
    pub href: Option<Href>,

    /// The actual STAC object.
    pub inner: Value,
}

/// Any STAC object, represented as an enum.
#[derive(Debug, PartialEq, Clone)]
pub enum Value {
    /// An [Item].
    Item(Item),

    /// A [Catalog].
    Catalog(Catalog),

    /// A [Collection].
    Collection(Collection),
}

impl Object {
    /// Creates a new object with an href.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Object, Item, Href};
    /// let item = Item::new("an-id");
    /// let object = Object::new(item, "an-href").unwrap();
    /// assert_eq!(object.href.as_ref().unwrap().as_str(), "an-href");
    /// assert!(object.is_item());
    /// ```
    pub fn new<O, T>(object: O, href: T) -> Result<Object, Error>
    where
        O: Into<Value>,
        T: Into<Href>,
    {
        Ok(Object {
            href: Some(href.into()),
            inner: object.into(),
        })
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
    pub fn from_value(value: serde_json::Value) -> Result<Object, Error> {
        if let Some(type_) = value.get(TYPE_FIELD) {
            if let Some(type_) = type_.as_str() {
                match type_ {
                    ITEM_TYPE => Ok(Object {
                        inner: Value::Item(serde_json::from_value(value)?),
                        href: None,
                    }),
                    CATALOG_TYPE => Ok(Object {
                        inner: Value::Catalog(serde_json::from_value(value)?),
                        href: None,
                    }),
                    COLLECTION_TYPE => Ok(Object {
                        inner: Value::Collection(serde_json::from_value(value)?),
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

    /// Returns true if this object is a [Catalog].
    ///
    /// # Examples
    ///
    /// ```
    /// let catalog = stac::read("data/catalog.json").unwrap();
    /// assert!(catalog.is_catalog());
    /// ```
    pub fn is_catalog(&self) -> bool {
        matches!(self.inner, Value::Catalog(_))
    }

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
            Value::Catalog(catalog) => Some(catalog),
            _ => None,
        }
    }

    /// Returns a mutable reference to this object as a [Catalog], or None if it is not a Catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut catalog = stac::read("data/catalog.json").unwrap();
    /// catalog.as_mut_catalog().unwrap().description = "a new description".to_string();
    /// ```
    pub fn as_mut_catalog(&mut self) -> Option<&mut Catalog> {
        match &mut self.inner {
            Value::Catalog(catalog) => Some(catalog),
            _ => None,
        }
    }

    /// Returns true if this object is a [Collection].
    ///
    /// # Examples
    ///
    /// ```
    /// let collection = stac::read("data/collection.json").unwrap();
    /// assert!(collection.is_collection());
    /// ```
    pub fn is_collection(&self) -> bool {
        matches!(self.inner, Value::Collection(_))
    }

    /// Returns a reference to this object as a [Collection], or None if it is not a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// let collection = stac::read("data/collection.json").unwrap();
    /// println!("Description: {}", collection.as_collection().unwrap().description);
    /// ```
    pub fn as_collection(&self) -> Option<&Collection> {
        match &self.inner {
            Value::Collection(collection) => Some(collection),
            _ => None,
        }
    }

    /// Returns a reference to this object as a [Collection], or None if it is not a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut collection = stac::read("data/collection.json").unwrap();
    /// collection.as_mut_collection().unwrap().description = "a new description".to_string();
    /// ```
    pub fn as_mut_collection(&mut self) -> Option<&mut Collection> {
        match &mut self.inner {
            Value::Collection(collection) => Some(collection),
            _ => None,
        }
    }

    /// Returns true if this object is an [Item].
    ///
    /// # Examples
    ///
    /// ```
    /// let item = stac::read("data/simple-item.json").unwrap();
    /// assert!(item.is_item());
    /// ```
    pub fn is_item(&self) -> bool {
        matches!(self.inner, Value::Item(_))
    }

    /// Returns a reference to this object as an [Item], or None if it is not an item.
    ///
    /// # Examples
    ///
    /// ```
    /// let item = stac::read("data/simple-item.json").unwrap();
    /// println!("Collection: {}", item.as_item().unwrap().collection.as_ref().unwrap());
    /// ```
    pub fn as_item(&self) -> Option<&Item> {
        match &self.inner {
            Value::Item(item) => Some(item),
            _ => None,
        }
    }

    /// Returns a mutable reference to this object as an [Item], or None if it is not an item.
    ///
    /// # Examples
    ///
    /// ```
    /// let mut item = stac::read("data/simple-item.json").unwrap();
    /// item.as_mut_item().unwrap().collection = Some("a-new-collection".to_string());
    /// ```
    pub fn as_mut_item(&mut self) -> Option<&mut Item> {
        match &mut self.inner {
            Value::Item(item) => Some(item),
            _ => None,
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
            Value::Item(item) => &item.id,
            Value::Catalog(catalog) => &catalog.id,
            Value::Collection(collection) => &collection.id,
        }
    }

    /// Returns a reference to this object's title.
    ///
    /// For [Items](Item), this checks for a `title` field in the
    /// `additional_fields` attribute and returns it as a string if possible.
    ///
    /// # Examples
    ///
    /// ```
    /// let catalog = stac::read("data/catalog.json").unwrap();
    /// assert_eq!(catalog.title().unwrap(), "Example Catalog");
    /// ```
    pub fn title(&self) -> Option<&str> {
        match &self.inner {
            Value::Item(item) => item
                .additional_fields
                .get("title")
                .and_then(|value| value.as_str()),
            Value::Catalog(catalog) => catalog.title.as_deref(),
            Value::Collection(collection) => collection.title.as_deref(),
        }
    }

    /// Returns a reference to this object's links.
    ///
    /// # Examples
    ///
    /// ```
    /// let object = stac::read("data/catalog.json").unwrap();
    /// let links = object.links();
    /// assert_eq!(links.len(), 6);
    /// ```
    pub fn links(&self) -> &[Link] {
        match &self.inner {
            Value::Item(item) => &item.links,
            Value::Catalog(catalog) => &catalog.links,
            Value::Collection(collection) => &collection.links,
        }
    }

    /// Adds a link to this object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "a-rel");
    /// let mut object = stac::read("data/catalog.json").unwrap();
    /// object.add_link(link);
    /// ```
    pub fn add_link(&mut self, link: Link) {
        match &mut self.inner {
            Value::Item(item) => item.links.push(link),
            Value::Catalog(catalog) => catalog.links.push(link),
            Value::Collection(collection) => collection.links.push(link),
        }
    }

    /// Converts this object into a [serde_json::Value].
    ///
    /// The href gets dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// let object = stac::read("data/catalog.json").unwrap();
    /// let value = object.into_value().unwrap();
    /// ```
    pub fn into_value(self) -> Result<serde_json::Value, Error> {
        match self.inner {
            Value::Item(item) => serde_json::to_value(item).map_err(Error::from),
            Value::Catalog(catalog) => serde_json::to_value(catalog).map_err(Error::from),
            Value::Collection(collection) => serde_json::to_value(collection).map_err(Error::from),
        }
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
            Value::Item(item) => Some(item),
            _ => None,
        }
    }

    /// Returns this object's inner [Catalog], or `None` if it is not a catalog.
    ///
    /// # Examples
    ///
    /// ```
    /// let catalog = stac::read("data/catalog.json").unwrap();
    /// let catalog = catalog.into_catalog().unwrap();
    /// ```
    pub fn into_catalog(self) -> Option<Catalog> {
        match self.inner {
            Value::Catalog(catalog) => Some(catalog),
            _ => None,
        }
    }

    /// Returns this object's inner [Collection], or `None` if it is not a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// let collection = stac::read("data/collection.json").unwrap();
    /// let collection = collection.into_collection().unwrap();
    /// ```
    pub fn into_collection(self) -> Option<Collection> {
        match self.inner {
            Value::Collection(collection) => Some(collection),
            _ => None,
        }
    }

    /// Removes all [structural links](Link::is_structural) from this object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let mut catalog = stac::read("data/catalog.json").unwrap();
    /// catalog.add_link(Link::new("an-href", "not-structural"));
    /// catalog.remove_structural_links();
    /// assert_eq!(catalog.links().len(), 1);
    /// ```
    pub fn remove_structural_links(&mut self) {
        let f = |link: &Link| !link.is_structural();
        match &mut self.inner {
            Value::Item(item) => item.links.retain(f),
            Value::Catalog(catalog) => catalog.links.retain(f),
            Value::Collection(collection) => collection.links.retain(f),
        }
    }
}

impl From<Catalog> for Value {
    fn from(catalog: Catalog) -> Value {
        Value::Catalog(catalog)
    }
}

impl From<Collection> for Value {
    fn from(collection: Collection) -> Value {
        Value::Collection(collection)
    }
}

impl From<Item> for Value {
    fn from(item: Item) -> Value {
        Value::Item(item)
    }
}

impl<T: Into<Value>> From<T> for Object {
    fn from(object: T) -> Object {
        Object {
            href: None,
            inner: object.into(),
        }
    }
}
