//! Validate STAC objects with [json-schema](https://json-schema.org/).
//!
//! If the `jsonschema` feature is enabled, STAC objects can be validated using [jsonschema](https://docs.rs/jsonschema/latest/jsonschema/).
//! Objects are validated against schema definitions complied into the library, but network access is still required for associated schemas (e.g. GeoJSON) and extension schemas; the `jsonschema` feature also enables `reqwest`.
//! There's an example in the `examples/` directory of the repo that demonstrates how you might use the validator in a command-line utility.
//! Run it like this:
//!
//! ```shell
//! cargo run --example validate --feature jsonschema example/invalid-item.json
//! ```
//!
//! # Examples
//!
//! The [Validate] trait adds a `.validate()` method to all three objects types and [Value].
//! Errors are returned as a vector of errors.
//!
//! ```
//! use stac::Validate;
//! stac::read("data/simple-item.json").unwrap().validate().unwrap();
//! let errors = stac::read("examples/invalid-item.json").unwrap().validate().unwrap_err();
//! for error in errors {
//!     println!("ERROR: {}", error);
//! }
//! ```
//!
//! If you're doing multiple validations, it is more efficient to use the [Validator] structure, which will cache any fetched schemas, including extension schemas:
//!
//! ```
//! use stac::Validator;
//! let mut validator = Validator::new().unwrap();
//! let item = stac::read("data/simple-item.json").unwrap();
//! let catalog = stac::read("data/catalog.json").unwrap();
//! validator.validate(item).unwrap();
//! validator.validate(catalog).unwrap();
//! ```

use crate::{Catalog, Collection, Error, Extensions, Item, ItemCollection, Value};
use jsonschema::{JSONSchema, ValidationError};
use serde::Serialize;
use std::{borrow::Cow, collections::HashMap};

/// A structure that performs json-schema validations.
///
/// Includes pre-compiled schemas for all three STAC object types, as well as a cache for extension schemas.
#[derive(Debug)]
pub struct Validator {
    item_schema: JSONSchema,
    catalog_schema: JSONSchema,
    collection_schema: JSONSchema,
    extension_schemas: HashMap<String, JSONSchema>,
}

/// A trait to provide validation on STAC objects.
pub trait Validate {
    /// Validate this STAC object using a one-time-use [Validator].
    ///
    /// Validation consumes the object.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Validate, Item};
    /// let item = Item::new("an-id");
    /// item.validate().unwrap(); // <- item is consumed
    /// ```
    fn validate(self) -> Result<(), Vec<Error>>;
}

enum Schema {
    Item,
    Collection,
    Catalog,
}

impl Validator {
    /// Creates a new validator.
    ///
    /// # Examples
    ///
    /// ```
    /// let validator = stac::Validator::new().unwrap();
    /// ```
    pub fn new() -> Result<Validator, Error> {
        // TODO support falling back to network fetch for unknown versions.
        Ok(Validator {
            item_schema: compile_schema(include_str!("../schemas/v1.0.0/item.json"))?,
            catalog_schema: compile_schema(include_str!("../schemas/v1.0.0/catalog.json"))?,
            collection_schema: compile_schema(include_str!("../schemas/v1.0.0/collection.json"))?,
            extension_schemas: HashMap::new(),
        })
    }

    /// Validate an [Item].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Item, Validator};
    /// let item = Item::new("an-id");
    /// let mut validator = Validator::new().unwrap();
    /// validator.validate_item(item).unwrap();
    /// ```
    pub fn validate_item(&mut self, item: Item) -> Result<(), Vec<Error>> {
        self.validate_with_schema(Schema::Item, item)
    }

    /// Validate a [Catalog].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Catalog, Validator};
    /// let catalog = Catalog::new("an-id", "a description");
    /// let mut validator = Validator::new().unwrap();
    /// validator.validate_catalog(catalog).unwrap();
    /// ```
    pub fn validate_catalog(&mut self, catalog: Catalog) -> Result<(), Vec<Error>> {
        self.validate_with_schema(Schema::Catalog, catalog)
    }

    /// Validate a [Collection].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Collection, Validator};
    /// let collection = Collection::new("an-id", "a description");
    /// let mut validator = Validator::new().unwrap();
    /// validator.validate_collection(collection).unwrap();
    /// ```
    pub fn validate_collection(&mut self, collection: Collection) -> Result<(), Vec<Error>> {
        self.validate_with_schema(Schema::Collection, collection)
    }

    /// Validate an [ItemCollection].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{ItemCollection, Item, Validator};
    /// let item_collection: ItemCollection = vec![Item::new("a"), Item::new("b")].into();
    /// let mut validator = Validator::new().unwrap();
    /// validator.validate_item_collection(item_collection).unwrap();
    /// ```
    pub fn validate_item_collection(
        &mut self,
        item_collection: ItemCollection,
    ) -> Result<(), Vec<Error>> {
        let mut errors = Vec::new();
        for item in item_collection.items {
            if let Err(e) = self.validate_item(item) {
                errors.extend(e);
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Validate a [Value].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Value, Validator};
    /// let mut validator = Validator::new().unwrap();
    /// let item = stac::read("data/simple-item.json").unwrap();
    /// validator.validate(item).unwrap();
    /// ```
    pub fn validate(&mut self, value: Value) -> Result<(), Vec<Error>> {
        match value {
            Value::Item(item) => self.validate_item(item),
            Value::Catalog(catalog) => self.validate_catalog(catalog),
            Value::Collection(collection) => self.validate_collection(collection),
            Value::ItemCollection(item_collection) => {
                self.validate_item_collection(item_collection)
            }
        }
    }

    fn validate_with_schema<V: Serialize + Extensions>(
        &mut self,
        schema: Schema,
        value: V,
    ) -> Result<(), Vec<Error>> {
        let extension_schemas = if let Some(extensions) = value.extensions() {
            for extension in extensions {
                self.ensure_extension_schema(extension)
                    .map_err(|e| vec![e])?;
            }
            Some(
                extensions
                    .iter()
                    .map(|extension| self.extension_schemas.get(extension).unwrap())
                    .collect::<Vec<_>>(),
            )
        } else {
            None
        };
        let schema = match schema {
            Schema::Item => &self.item_schema,
            Schema::Catalog => &self.catalog_schema,
            Schema::Collection => &self.collection_schema,
        };

        let mut errors = Vec::new();
        let value = serde_json::to_value(value).map_err(|e| vec![Error::from(e)])?;
        if let Err(e) = schema.validate(&value).map_err(|iter| iter.map(into_error)) {
            errors.extend(e);
        }
        if let Some(extension_schemas) = extension_schemas {
            for schema in extension_schemas {
                if let Err(e) = schema.validate(&value).map_err(|iter| iter.map(into_error)) {
                    errors.extend(e);
                }
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    fn ensure_extension_schema(&mut self, extension: &str) -> Result<(), Error> {
        if self.extension_schemas.contains_key(extension) {
            return Ok(());
        }
        let value = crate::read_json(extension)?;
        let schema = JSONSchema::compile(&value).map_err(into_error)?;
        let _ = self.extension_schemas.insert(extension.to_string(), schema);
        Ok(())
    }
}

impl Validate for Item {
    fn validate(self) -> Result<(), Vec<Error>> {
        Validator::new()
            .map_err(|e| vec![e])
            .and_then(|mut v| v.validate_item(self))
    }
}

impl Validate for Catalog {
    fn validate(self) -> Result<(), Vec<Error>> {
        Validator::new()
            .map_err(|e| vec![e])
            .and_then(|mut v| v.validate_catalog(self))
    }
}

impl Validate for Collection {
    fn validate(self) -> Result<(), Vec<Error>> {
        Validator::new()
            .map_err(|e| vec![e])
            .and_then(|mut v| v.validate_collection(self))
    }
}

impl Validate for ItemCollection {
    fn validate(self) -> Result<(), Vec<Error>> {
        Validator::new()
            .map_err(|e| vec![e])
            .and_then(|mut v| v.validate_item_collection(self))
    }
}

impl Validate for Value {
    fn validate(self) -> Result<(), Vec<Error>> {
        match self {
            Value::Item(item) => item.validate(),
            Value::Catalog(catalog) => catalog.validate(),
            Value::Collection(collection) => collection.validate(),
            Value::ItemCollection(item_collection) => item_collection.validate(),
        }
    }
}

fn compile_schema(s: &str) -> Result<JSONSchema, Error> {
    let schema = serde_json::from_str(s)?;
    JSONSchema::compile(&schema).map_err(into_error)
}

fn into_error(validation_error: ValidationError<'_>) -> Error {
    Error::from(ValidationError {
        instance_path: validation_error.instance_path.clone(),
        instance: Cow::Owned(validation_error.instance.into_owned()),
        kind: validation_error.kind,
        schema_path: validation_error.schema_path,
    })
}

#[cfg(test)]
mod tests {
    use super::Validate;
    use crate::{Catalog, Collection, Item};

    #[test]
    fn valid_item() {
        let item = Item::new("an-id");
        item.validate().unwrap();
    }

    #[test]
    fn invalid_item() {
        let mut item = Item::new("an-id");
        item.id = String::new();
        let errors = item.validate().unwrap_err();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn valid_catalog() {
        let catalog = Catalog::new("an-id", "a description");
        catalog.validate().unwrap();
    }

    #[test]
    fn invalid_catalog() {
        let mut catalog = Catalog::new("an-id", "a description");
        catalog.id = String::new();
        let errors = catalog.validate().unwrap_err();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn valid_collection() {
        let collection = Collection::new("an-id", "a description");
        collection.validate().unwrap();
    }

    #[test]
    fn invalid_collection() {
        let mut collection = Collection::new("an-id", "a description");
        collection.id = String::new();
        let errors = collection.validate().unwrap_err();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn valid_extension() {
        let mut item = Item::new("an-id");
        // TODO once https://github.com/gadomski/stac-rs/issues/35 is fixed, use that mechanism
        let _ = item
            .properties
            .additional_fields
            .insert("proj:epsg".to_string(), 4326.into());
        item.extensions = Some(vec![
            "https://stac-extensions.github.io/projection/v1.0.0/schema.json".to_string(),
        ]);
        item.validate().unwrap();
    }

    #[test]
    fn invalid_extension() {
        let mut item = Item::new("an-id");
        // TODO once https://github.com/gadomski/stac-rs/issues/35 is fixed, use that mechanism
        let _ = item
            .properties
            .additional_fields
            .insert("proj:epsg".to_string(), "not an integer".into());
        item.extensions = Some(vec![
            "https://stac-extensions.github.io/projection/v1.0.0/schema.json".to_string(),
        ]);
        let errors = item.validate().unwrap_err();
        assert_eq!(errors.len(), 1);
    }

    #[test]
    fn item_collection() {
        let item_collection = crate::read("examples/item-collection.json").unwrap();
        item_collection.validate().unwrap();
    }
}
