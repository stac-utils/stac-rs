use crate::{Error, Result, Validate};
use jsonschema::{JSONSchema, SchemaResolver, SchemaResolverError};
use serde_json::Value;
use std::{
    collections::HashMap,
    sync::Arc,
    thread::{self, JoinHandle},
};
use url::Url;

/// A structure for validating one or more STAC objects.
///
/// The structure stores the core json schemas, and has a cache for extension
/// schemas.
#[derive(Debug)]
pub struct Validator {
    item_schema: JSONSchema,
    catalog_schema: JSONSchema,
    collection_schema: JSONSchema,
    extension_schemas: HashMap<String, JSONSchema>,
}

#[derive(Default)]
struct Resolver;

impl Validator {
    /// Creates a new validator.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_validate::Validator;
    /// let validator = Validator::new();
    /// ```
    pub fn new() -> Validator {
        let mut options = JSONSchema::options();
        let options = options.with_resolver(Resolver);
        Validator {
            item_schema: options
                .compile(
                    &serde_json::from_str(include_str!("../schemas/v1.0.0/item.json")).unwrap(),
                )
                .unwrap(),
            catalog_schema: options
                .compile(
                    &serde_json::from_str(include_str!("../schemas/v1.0.0/catalog.json")).unwrap(),
                )
                .unwrap(),
            collection_schema: options
                .compile(
                    &serde_json::from_str(include_str!("../schemas/v1.0.0/collection.json"))
                        .unwrap(),
                )
                .unwrap(),
            extension_schemas: HashMap::new(),
        }
    }

    /// Validates a STAC object.
    ///
    /// Needs to be mutable because extension schemas are cached.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_validate::Validator;
    /// let mut validator = Validator::new();
    /// validator.validate(stac::Item::new("an-id")).unwrap();
    /// ```
    pub fn validate<V: Validate>(&mut self, validatable: V) -> Result<()> {
        validatable.validate_with_validator(self)
    }

    pub(crate) fn validate_item(&self, item: &Value) -> Result<()> {
        self.item_schema
            .validate(item)
            .map_err(Error::from_validation_errors)
    }

    pub(crate) fn validate_catalog(&self, item: &Value) -> Result<()> {
        self.catalog_schema
            .validate(item)
            .map_err(Error::from_validation_errors)
    }

    pub(crate) fn validate_collection(&self, collection: &Value) -> Result<()> {
        self.collection_schema
            .validate(collection)
            .map_err(Error::from_validation_errors)
    }

    pub(crate) fn validate_item_collection(&self, item_collection: &Value) -> Result<()> {
        if let Some(items) = item_collection.get("features") {
            if let Some(items) = items.as_array() {
                let mut errors = Vec::new();
                for item in items {
                    match self.validate_item(item) {
                        Ok(()) => {}
                        Err(Error::Validation(e)) => errors.extend(e),
                        Err(e) => return Err(e),
                    }
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(Error::Validation(errors))
                }
            } else {
                // FIXME these errors aren't quite correct but there's probably
                // a refactor to be done around getting the items from a
                // serde_json::Value feature collection.
                Err(stac::Error::UnknownType("features".to_string()).into())
            }
        } else {
            Err(stac::Error::MissingType.into())
        }
    }

    pub(crate) fn validate_extensions(&mut self, value: &Value) -> Result<()> {
        if let Some(extensions) = value.get("stac_extensions") {
            if let Some(extensions) = extensions.as_array() {
                let mut errors = Vec::new();
                let mut handles: Vec<JoinHandle<_>> = Vec::new();
                for extension in extensions {
                    if let Some(extension) = extension.as_str() {
                        if let Some(schema) = self.extension_schemas.get(extension) {
                            match self.validate_extension(value, schema) {
                                Ok(()) => {}
                                Err(Error::Validation(extension_errors)) => {
                                    errors.extend(extension_errors)
                                }
                                Err(error) => {
                                    for handle in handles {
                                        let _ = handle.join().unwrap();
                                    }
                                    return Err(error);
                                }
                            }
                        } else {
                            let extension = extension.to_string();
                            handles.push(thread::spawn(move || get_extension(extension)));
                        }
                    } else {
                        for handle in handles {
                            let _ = handle.join().unwrap();
                        }
                        return Err(Error::IncorrectStacExtensionsType(extension.clone()));
                    }
                }
                for handle in handles {
                    let (href, schema) = handle.join().unwrap()?;
                    match self.validate_extension(value, &schema) {
                        Ok(()) => {}
                        Err(Error::Validation(extension_errors)) => errors.extend(extension_errors),
                        Err(error) => {
                            return Err(error);
                        }
                    }
                    let _ = self.extension_schemas.insert(href, schema);
                }
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(Error::Validation(errors))
                }
            } else {
                Err(Error::IncorrectStacExtensionsType(extensions.clone()))
            }
        } else {
            Ok(())
        }
    }

    fn validate_extension(&self, value: &Value, extension: &JSONSchema) -> Result<()> {
        extension
            .validate(value)
            .map_err(Error::from_validation_errors)
    }
}

impl Default for Validator {
    fn default() -> Self {
        Self::new()
    }
}

impl SchemaResolver for Resolver {
    fn resolve(
        &self,
        _: &Value,
        url: &Url,
        _: &str,
    ) -> std::result::Result<Arc<Value>, SchemaResolverError> {
        match url.as_str() {
            "https://geojson.org/schema/Feature.json" => Ok(Arc::new(
                serde_json::from_str(include_str!("../schemas/geojson/Feature.json")).unwrap(),
            )),
            "https://geojson.org/schema/Geometry.json" => Ok(Arc::new(
                serde_json::from_str(include_str!("../schemas/geojson/Geometry.json")).unwrap(),
            )),
            "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/basics.json" => {
                Ok(Arc::new(
                    serde_json::from_str(include_str!("../schemas/v1.0.0/basics.json")).unwrap(),
                ))
            }
            "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/datetime.json" => {
                Ok(Arc::new(
                    serde_json::from_str(include_str!("../schemas/v1.0.0/datetime.json")).unwrap(),
                ))
            }
            "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/instrument.json" => {
                Ok(Arc::new(
                    serde_json::from_str(include_str!("../schemas/v1.0.0/instrument.json"))
                        .unwrap(),
                ))
            }
            "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/licensing.json" => {
                Ok(Arc::new(
                    serde_json::from_str(include_str!("../schemas/v1.0.0/licensing.json")).unwrap(),
                ))
            }
            "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/provider.json" => {
                Ok(Arc::new(
                    serde_json::from_str(include_str!("../schemas/v1.0.0/provider.json")).unwrap(),
                ))
            }
            _ => match url.scheme() {
                "http" | "https" => {
                    let response = reqwest::blocking::get(url.as_str())?;
                    let document: Value = response.json()?;
                    Ok(Arc::new(document))
                }
                "file" => {
                    if let Ok(path) = url.to_file_path() {
                        let f = std::fs::File::open(path)?;
                        let document: Value = serde_json::from_reader(f)?;
                        Ok(Arc::new(document))
                    } else {
                        Err(Error::InvalidFilePath(url.clone()).into())
                    }
                }
                "json-schema" => Err(Error::CannotResolveJsonSchemaScheme(url.clone()).into()),
                _ => Err(Error::InvalidUrlScheme(url.clone()).into()),
            },
        }
    }
}

fn get_extension(href: String) -> Result<(String, JSONSchema)> {
    let response = reqwest::blocking::get(&href)?;
    let response = response.error_for_status()?;
    let json: Value = response.json()?;
    let mut options = JSONSchema::options();
    let options = options.with_resolver(Resolver);
    let schema = options
        .compile(&json)
        .map_err(Error::from_validation_error)?;
    Ok((href, schema))
}
