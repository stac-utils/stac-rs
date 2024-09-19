use crate::{Error, Extension, Result};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{json, Map, Value};

/// Trait for structures that have gettable and settable fields.
///
/// For most structures in this crate, this will be the `additional_properties`
/// attribute. The primary exception is [Item](crate::Item), which gets and sets
/// additional fields in its `properties` attribute.
///
/// # Examples
///
/// ```
/// use stac::{Catalog, Item, Fields};
///
/// let mut catalog = Catalog::new("an-id", "a description");
/// catalog.set_field("foo", "bar");
/// assert_eq!(catalog.additional_fields.get("foo").unwrap(), "bar");
///
/// let mut item = Item::new("an-id");
/// item.set_field("foo", "bar");
/// assert_eq!(item.properties.additional_fields.get("foo").unwrap(), "bar");
/// assert!(item.additional_fields.is_empty());
/// ```
pub trait Fields {
    /// Gets the fields value.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Fields};
    /// let item = Item::new("an-id");
    /// assert!(item.fields().is_empty());
    /// ```
    fn fields(&self) -> &Map<String, Value>;

    /// Gets a mutable reference to the fields value.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Fields};
    /// let mut item = Item::new("an-id");
    /// item.fields_mut().insert("foo".to_string(), "bar".into());
    /// ```
    fn fields_mut(&mut self) -> &mut Map<String, Value>;

    /// Gets the value of a field.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Fields};
    /// let mut item = Item::new("an-id");
    /// item.set_field("foo", "bar").unwrap();
    /// assert_eq!(item.properties.additional_fields.get("foo"), item.field("foo"));
    /// ```
    fn field(&self, key: &str) -> Option<&Value> {
        self.fields().get(key)
    }

    /// Sets the value of a field.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Fields};
    /// let mut item = Item::new("an-id");
    /// item.set_field("foo", "bar").unwrap();
    /// assert_eq!(item.properties.additional_fields["foo"], "bar");
    /// ```
    fn set_field<S: Serialize>(&mut self, key: impl ToString, value: S) -> Result<Option<Value>> {
        let value = serde_json::to_value(value)?;
        Ok(self.fields_mut().insert(key.to_string(), value))
    }

    /// Gets values with a prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Fields, Item, extensions::Projection};
    /// let item: Item = stac::read("examples/extensions-collection/proj-example/proj-example.json").unwrap();
    /// let projection: Projection = item.fields_with_prefix("proj").unwrap();  // Prefer `Extensions::extension`
    /// ```
    fn fields_with_prefix<D: DeserializeOwned>(&self, prefix: &str) -> Result<D> {
        let mut map = Map::new();
        let prefix = format!("{}:", prefix);
        for (key, value) in self.fields().iter() {
            if key.starts_with(&prefix) && key.len() > prefix.len() {
                let _ = map.insert(key[prefix.len()..].to_string(), value.clone());
            }
        }
        serde_json::from_value(json!(map)).map_err(Error::from)
    }

    /// Sets values with a prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Fields, Item, extensions::Projection};
    /// let projection = Projection { code: Some("EPSG:4326".to_string()), ..Default::default() };
    /// let mut item = Item::new("an-id");
    /// item.set_fields_with_prefix("proj", projection);  // Prefer `Extensions::set_extension`
    /// ```
    fn set_fields_with_prefix<S: Serialize>(&mut self, prefix: &str, value: S) -> Result<()> {
        let value = serde_json::to_value(value)?;
        if let Value::Object(object) = value {
            for (key, value) in object.into_iter() {
                let _ = self.set_field(format!("{}:{}", prefix, key), value);
            }
            Ok(())
        } else {
            Err(Error::NotAnObject(value))
        }
    }

    /// Removes values with a prefix.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Fields, Item, extensions::Projection};
    /// let projection = Projection { code: Some("EPSG:4326".to_string()), ..Default::default() };
    /// let mut item = Item::new("an-id");
    /// item.remove_fields_with_prefix("proj");  // Prefer `Fields::remove_extension`
    /// ```
    fn remove_fields_with_prefix(&mut self, prefix: &str) {
        let prefix = format!("{}:", prefix);
        self.fields_mut()
            .retain(|key, _| !(key.starts_with(&prefix) && key.len() > prefix.len()));
    }

    /// Gets an extension's data.
    ///
    /// Returns `Ok(None)` if the object doesn't have the given extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Fields, extensions::Projection};
    /// let item: Item = stac::read("examples/extensions-collection/proj-example/proj-example.json").unwrap();
    /// let projection: Projection = item.extension().unwrap();
    /// assert_eq!(projection.code.unwrap(), "EPSG:32614");
    /// ```
    fn extension<E: Extension>(&self) -> Result<E> {
        self.fields_with_prefix(E::PREFIX)
    }

    /// Sets an extension's data into this object.
    ///
    /// This will remove any previous fields from this extension
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Fields, extensions::Projection};
    /// let mut item = Item::new("an-id");
    /// let projection = Projection { code: Some("EPSG:4326".to_string()), ..Default::default() };
    /// item.set_extension(projection).unwrap();
    /// ```
    fn set_extension<E: Extension>(&mut self, extension: E) -> Result<()> {
        self.remove_extension::<E>();
        self.set_fields_with_prefix(E::PREFIX, extension)
    }

    /// Removes all of the extension's fields from this object.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, extensions::{Projection, Extensions}};
    /// let mut item: Item = stac::read("examples/extensions-collection/proj-example/proj-example.json").unwrap();
    /// assert!(item.has_extension::<Projection>());
    /// item.remove_extension::<Projection>();
    /// assert!(!item.has_extension::<Projection>());
    /// ```
    fn remove_extension<E: Extension>(&mut self) {
        self.remove_fields_with_prefix(E::PREFIX);
    }
}
