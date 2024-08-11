use crate::{Error, Result, Validate};
use jsonschema::{CompilationOptions, JSONSchema, SchemaResolver, SchemaResolverError};
use serde_json::Value;
use stac::Version;
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
    item_schemas: HashMap<Version, JSONSchema>,
    catalog_schemas: HashMap<Version, JSONSchema>,
    collection_schemas: HashMap<Version, JSONSchema>,
    extension_schemas: HashMap<String, JSONSchema>,
    compilation_options: CompilationOptions,
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
        let options_with_resolver = options.with_resolver(Resolver);
        let mut item_schemas = HashMap::new();
        let _ = item_schemas.insert(
            Version::v1_0_0,
            options_with_resolver
                .compile(
                    &serde_json::from_str(include_str!("../schemas/v1.0.0/item.json")).unwrap(),
                )
                .unwrap(),
        );
        let mut catalog_schemas = HashMap::new();
        let _ = catalog_schemas.insert(
            Version::v1_0_0,
            options_with_resolver
                .compile(
                    &serde_json::from_str(include_str!("../schemas/v1.0.0/catalog.json")).unwrap(),
                )
                .unwrap(),
        );
        let mut collection_schemas = HashMap::new();
        let _ = collection_schemas.insert(
            Version::v1_0_0,
            options_with_resolver
                .compile(
                    &serde_json::from_str(include_str!("../schemas/v1.0.0/collection.json"))
                        .unwrap(),
                )
                .unwrap(),
        );
        Validator {
            item_schemas,
            catalog_schemas,
            collection_schemas,
            extension_schemas: HashMap::new(),
            compilation_options: options,
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

    pub(crate) fn validate_item(&mut self, item: &Value) -> Result<()> {
        let version = version(item)?;
        self.item_schema_for(version)?
            .validate(item)
            .map_err(Error::from_validation_errors)
    }

    fn item_schema_for(&mut self, version: Version) -> Result<&JSONSchema> {
        if !self.item_schemas.contains_key(&version) {
            let schema = self.fetch_schema(&format!(
                "https://schemas.stacspec.org/v{}/item-spec/json-schema/item.json",
                version
            ))?;
            let _ = self.item_schemas.insert(version, schema);
        }
        Ok(self
            .item_schemas
            .get(&version)
            .expect("if we didn't have it, we should have just fetched it"))
    }

    pub(crate) fn validate_catalog(&mut self, catalog: &Value) -> Result<()> {
        let version = version(catalog)?;
        self.catalog_schema_for(version)?
            .validate(catalog)
            .map_err(Error::from_validation_errors)
    }

    fn catalog_schema_for(&mut self, version: Version) -> Result<&JSONSchema> {
        if !self.catalog_schemas.contains_key(&version) {
            let schema = self.fetch_schema(&format!(
                "https://schemas.stacspec.org/v{}/catalog-spec/json-schema/catalog.json",
                version
            ))?;
            let _ = self.catalog_schemas.insert(version, schema);
        }
        Ok(self
            .catalog_schemas
            .get(&version)
            .expect("if we didn't have it, we should have just fetched it"))
    }

    pub(crate) fn validate_collection(&mut self, collection: &Value) -> Result<()> {
        let version = version(collection)?;
        self.collection_schema_for(version)?
            .validate(collection)
            .map_err(Error::from_validation_errors)
    }

    fn collection_schema_for(&mut self, version: Version) -> Result<&JSONSchema> {
        if !self.collection_schemas.contains_key(&version) {
            let schema = self.fetch_schema(&format!(
                "https://schemas.stacspec.org/v{}/collection-spec/json-schema/collection.json",
                version
            ))?;
            let _ = self.collection_schemas.insert(version, schema);
        }
        Ok(self
            .collection_schemas
            .get(&version)
            .expect("if we didn't have it, we should have just fetched it"))
    }

    fn fetch_schema(&mut self, url: &str) -> Result<JSONSchema> {
        let response = reqwest::blocking::get(url)?;
        let value: Value = response.json()?;
        let options = self.compilation_options.with_resolver(Resolver);
        options
            .compile(&value)
            .map_err(|_| Error::InvalidSchema(url.to_string()))
    }

    pub(crate) fn validate_item_collection(&mut self, item_collection: &Value) -> Result<()> {
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

fn version(value: &Value) -> Result<Version> {
    value
        .as_object()
        .and_then(|object| object.get("stac_version"))
        .and_then(|value| value.as_str())
        .map(|version| version.parse())
        .transpose()?
        .ok_or(Error::MissingStacVersion)
}
