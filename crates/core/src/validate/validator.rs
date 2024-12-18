use crate::{Error, Result, Type, Version};
use jsonschema::{Retrieve, Uri, ValidationOptions, Validator as JsonschemaValidator};
use reqwest::Client;
use serde::Serialize;
use serde_json::{Map, Value};
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::{sync::RwLock, task::JoinSet};

const SCHEMA_BASE: &str = "https://schemas.stacspec.org";

/// A cloneable structure for validating STAC.
#[derive(Clone, Debug)]
pub struct Validator {
    validation_options: ValidationOptions,
    cache: Arc<std::sync::RwLock<HashMap<Uri<String>, Value>>>,
    schemas: Arc<RwLock<HashMap<Uri<String>, Arc<JsonschemaValidator>>>>,
    uris: Arc<Mutex<HashSet<Uri<String>>>>,
    client: Client,
}

struct Retriever {
    cache: Arc<std::sync::RwLock<HashMap<Uri<String>, Value>>>,
    uris: Arc<Mutex<HashSet<Uri<String>>>>,
}

impl Validator {
    /// Creates a new validator.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Validator;
    ///
    /// # tokio_test::block_on(async {
    /// let validator = Validator::new().await.unwrap();
    /// });
    /// ```
    pub async fn new() -> Result<Validator> {
        let cache = Arc::new(std::sync::RwLock::new(cache()));
        let uris = Arc::new(Mutex::new(HashSet::new()));
        let retriever = Retriever {
            cache: cache.clone(),
            uris: uris.clone(),
        };
        let mut validation_options = JsonschemaValidator::options();
        let _ = validation_options.with_retriever(retriever);
        let client_builder = {
            #[cfg(feature = "reqwest-rustls")]
            {
                // Cloudflare can dislike when Github Actions requests stuff w/ the
                // default tls provider :shrug: so this is a workaround.
                Client::builder().use_rustls_tls()
            }
            #[cfg(not(feature = "reqwest-rustls"))]
            {
                Client::builder()
            }
        };
        let client_builder = client_builder.user_agent(concat!(
            env!("CARGO_PKG_NAME"),
            "/",
            env!("CARGO_PKG_VERSION"),
        ));
        Ok(Validator {
            schemas: Arc::new(RwLock::new(schemas(&validation_options))),
            validation_options,
            cache,
            uris,
            client: client_builder.build()?,
        })
    }

    /// Validates a single value.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Validator};
    ///
    /// let item = Item::new("an-id");
    /// # tokio_test::block_on(async {
    /// let validator = Validator::new().await.unwrap();
    /// validator.validate(&item).await.unwrap();
    /// });
    /// ```
    pub async fn validate<T>(&self, value: &T) -> Result<()>
    where
        T: Serialize,
    {
        let value = serde_json::to_value(value)?;
        if let Value::Object(object) = value {
            self.validate_object(object).await
        } else if let Value::Array(array) = value {
            self.validate_array(array).await
        } else {
            Err(Error::ScalarJson(value))
        }
    }

    fn validate_array(&self, array: Vec<Value>) -> Pin<Box<impl Future<Output = Result<()>>>> {
        // We have to pinbox because recursive async aren't allowed.
        let validator = self.clone();
        Box::pin(async move {
            let mut errors = Vec::new();
            for value in array {
                if let Err(error) = validator.validate(&value).await {
                    if let Error::Validation(e) = error {
                        errors.extend(e);
                    } else {
                        return Err(error);
                    }
                }
            }
            if errors.is_empty() {
                Ok(())
            } else {
                Err(Error::Validation(errors))
            }
        })
    }

    fn validate_object(
        &self,
        object: Map<String, Value>,
    ) -> Pin<Box<impl Future<Output = Result<()>>>> {
        // We have to pinbox because recursive async aren't allowed.
        let validator = self.clone();
        Box::pin(async move {
            let r#type = if let Some(r#type) = object.get("type").and_then(|v| v.as_str()) {
                let r#type: Type = r#type.parse()?;
                if r#type == Type::ItemCollection {
                    if let Some(features) = object.get("features") {
                        return validator.validate(features).await;
                    } else {
                        return Ok(());
                    }
                }
                r#type
            } else if let Some(collections) = object.get("collections").and_then(|v| v.as_array()) {
                return validator.validate(collections).await;
            } else {
                return Err(Error::MissingField("type"));
            };
            let version: Version = object
                .get("stac_version")
                .and_then(|v| v.as_str())
                .map(|v| v.parse::<Version>())
                .transpose()
                .unwrap()
                .ok_or(Error::MissingField("stac_version"))?;

            let url = build_schema_url(r#type, &version);
            let schema = validator.schema(url).await?;
            let value = Arc::new(Value::Object(object));
            let mut result = schema
                .validate(&value)
                .map_err(|e| Error::from_validation_errors(e, Some(&value)));
            if result.is_ok() {
                result = validator.validate_extensions(value.clone()).await
            }
            if let Err(err) = result {
                let mut join_set = JoinSet::new();
                {
                    let mut uris = validator.uris.lock().unwrap();
                    if uris.is_empty() {
                        return Err(err);
                    } else {
                        for url in uris.drain() {
                            let validator = validator.clone();
                            let _ = join_set.spawn(async move { validator.resolve(url).await });
                        }
                    }
                }
                while let Some(result) = join_set.join_next().await {
                    result??;
                }
                let object = if let Value::Object(o) = Arc::into_inner(value).unwrap() {
                    o
                } else {
                    unreachable!()
                };
                validator.validate_object(object).await
            } else {
                Ok(())
            }
        })
    }

    async fn validate_extensions(&self, value: Arc<Value>) -> Result<()> {
        let mut join_set = JoinSet::new();
        if let Some(extensions) = value
            .as_object()
            .and_then(|o| o.get("stac_extensions"))
            .and_then(|v| v.as_array())
        {
            for extension in extensions.iter().filter_map(|v| v.as_str()) {
                let extension = Uri::parse(extension)?.to_owned();
                let validator = self.clone();
                let value = value.clone();
                let _ = join_set.spawn(async move {
                    let extension_schema = validator.schema(extension).await?;
                    extension_schema
                        .validate(&value)
                        .map_err(|errors| Error::from_validation_errors(errors, Some(&value)))
                });
            }
        }
        let mut errors = Vec::new();
        while let Some(result) = join_set.join_next().await {
            let result = result?;
            if let Err(Error::Validation(e)) = result {
                errors.extend(e);
            } else if result.is_err() {
                return result;
            }
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Validation(errors))
        }
    }

    async fn schema(&self, uri: Uri<String>) -> Result<Arc<JsonschemaValidator>> {
        {
            let schemas = self.schemas.read().await;
            if let Some(schema) = schemas.get(&uri) {
                return Ok(schema.clone());
            }
        }
        self.resolve(uri.clone()).await?;
        let schema = {
            let cache = self.cache.read().unwrap();
            let value = cache.get(&uri).expect("we just resolved it");
            let schema = self.validation_options.build(value)?;
            Arc::new(schema)
        };
        {
            let mut schemas = self.schemas.write().await;
            let _ = schemas.insert(uri, schema.clone());
        }
        Ok(schema)
    }

    async fn resolve(&self, uri: Uri<String>) -> Result<()> {
        {
            let cache = self.cache.read().unwrap();
            if cache.contains_key(&uri) {
                return Ok(());
            }
        }
        tracing::debug!("resolving {}", uri);
        let value: Value = self
            .client
            .get(uri.as_str())
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        {
            let mut cache = self.cache.write().unwrap();
            let _ = cache.insert(uri, value);
        }
        Ok(())
    }
}

impl Retrieve for Retriever {
    fn retrieve(
        &self,
        uri: &Uri<&str>,
    ) -> std::result::Result<Value, Box<dyn std::error::Error + Send + Sync>> {
        let uri = uri.to_owned();
        {
            let cache = self.cache.read().unwrap();
            if let Some(schema) = cache.get(&uri) {
                return Ok(schema.clone());
            }
        }
        {
            let mut uris = self.uris.lock().unwrap();
            let _ = uris.insert(uri.clone());
        }
        Err(format!("{uri}").into())
    }
}

fn build_schema_url(r#type: Type, version: &Version) -> Uri<String> {
    Uri::parse(format!(
        "{}{}",
        SCHEMA_BASE,
        r#type
            .spec_path(version)
            .expect("we shouldn't get here with an item collection")
    ))
    .unwrap()
}

fn schemas(
    validation_options: &ValidationOptions,
) -> HashMap<Uri<String>, Arc<JsonschemaValidator>> {
    use Type::*;
    use Version::*;

    let mut schemas = HashMap::new();

    macro_rules! schema {
        ($t:expr, $v:expr, $path:expr, $schemas:expr) => {
            let url = build_schema_url($t, &$v);
            let schema = serde_json::from_str(include_str!($path)).unwrap();
            let schema = validation_options.build(&schema).unwrap();
            let _ = schemas.insert(url, Arc::new(schema));
        };
    }

    schema!(Item, v1_0_0, "schemas/v1.0.0/item.json", schemas);
    schema!(Catalog, v1_0_0, "schemas/v1.0.0/catalog.json", schemas);
    schema!(
        Collection,
        v1_0_0,
        "schemas/v1.0.0/collection.json",
        schemas
    );
    schema!(Item, v1_1_0, "schemas/v1.1.0/item.json", schemas);
    schema!(Catalog, v1_1_0, "schemas/v1.1.0/catalog.json", schemas);
    schema!(
        Collection,
        v1_1_0,
        "schemas/v1.1.0/collection.json",
        schemas
    );

    schemas
}

fn cache() -> HashMap<Uri<String>, Value> {
    let mut cache = HashMap::new();

    macro_rules! resolve {
        ($url:expr, $path:expr) => {
            let _ = cache.insert(
                Uri::parse($url.to_string()).unwrap(),
                serde_json::from_str(include_str!($path)).unwrap(),
            );
        };
    }

    // General
    resolve!(
        "https://geojson.org/schema/Feature.json",
        "schemas/geojson/Feature.json"
    );
    resolve!(
        "https://geojson.org/schema/Geometry.json",
        "schemas/geojson/Geometry.json"
    );
    resolve!(
        "http://json-schema.org/draft-07/schema",
        "schemas/json-schema/draft-07.json"
    );

    // STAC v1.0.0
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/basics.json",
        "schemas/v1.0.0/basics.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/datetime.json",
        "schemas/v1.0.0/datetime.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/instrument.json",
        "schemas/v1.0.0/instrument.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/item.json",
        "schemas/v1.0.0/item.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/licensing.json",
        "schemas/v1.0.0/licensing.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/provider.json",
        "schemas/v1.0.0/provider.json"
    );

    // STAC v1.1.0
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/bands.json",
        "schemas/v1.1.0/bands.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/basics.json",
        "schemas/v1.1.0/basics.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/common.json",
        "schemas/v1.1.0/common.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/data-values.json",
        "schemas/v1.1.0/data-values.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/datetime.json",
        "schemas/v1.1.0/datetime.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/instrument.json",
        "schemas/v1.1.0/instrument.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/item.json",
        "schemas/v1.1.0/item.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/licensing.json",
        "schemas/v1.1.0/licensing.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/provider.json",
        "schemas/v1.1.0/provider.json"
    );

    cache
}

#[cfg(test)]
mod tests {
    use serde_json::json;

    use super::Validator;
    use crate::{Collection, Item, Validate};

    #[tokio::test]
    async fn validate_array() {
        let items: Vec<_> = (0..100)
            .map(|i| Item::new(format!("item-{}", i)))
            .map(|i| serde_json::to_value(i).unwrap())
            .collect();
        let validator = Validator::new().await.unwrap();
        validator.validate(&items).await.unwrap();
    }

    #[tokio::test]
    async fn validate_collections() {
        let collection: Collection = crate::read("examples/collection.json").unwrap();
        let collections = json!({
            "collections": [collection]
        });
        collections.validate().await.unwrap();
    }
}
