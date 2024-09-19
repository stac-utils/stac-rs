use crate::{Error, Result};
use jsonschema::{
    SchemaResolver, SchemaResolverError, ValidationOptions, Validator as JsonschemaValidator,
};
use reqwest::Client;
use serde::Serialize;
use serde_json::{Map, Value};
use stac::{Type, Version};
use std::{
    collections::{HashMap, HashSet},
    future::Future,
    pin::Pin,
    sync::{Arc, Mutex},
};
use tokio::{
    sync::{
        mpsc::{error::TryRecvError, Receiver, Sender},
        oneshot::Sender as OneshotSender,
        RwLock,
    },
    task::JoinSet,
};
use url::Url;

const SCHEMA_BASE: &str = "https://schemas.stacspec.org";
const BUFFER: usize = 10;

/// A cloneable structure for validating STAC.
#[derive(Clone, Debug)]
pub struct Validator {
    validation_options: ValidationOptions,
    cache: Arc<std::sync::RwLock<HashMap<Url, Arc<Value>>>>,
    schemas: Arc<RwLock<HashMap<Url, Arc<JsonschemaValidator>>>>,
    urls: Arc<Mutex<HashSet<Url>>>,
    sender: Sender<(Url, OneshotSender<Result<Arc<Value>>>)>,
}

struct Resolver {
    cache: Arc<std::sync::RwLock<HashMap<Url, Arc<Value>>>>,
    urls: Arc<Mutex<HashSet<Url>>>,
}

impl Validator {
    /// Creates a new validator.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_validate::Validator;
    ///
    /// # tokio_test::block_on(async {
    /// let validator = Validator::new().await;
    /// });
    /// ```
    pub async fn new() -> Validator {
        let cache = Arc::new(std::sync::RwLock::new(cache()));
        let urls = Arc::new(Mutex::new(HashSet::new()));
        let resolver = Resolver {
            cache: cache.clone(),
            urls: urls.clone(),
        };
        let mut validation_options = JsonschemaValidator::options();
        let _ = validation_options.with_resolver(resolver);
        let (sender, receiver) = tokio::sync::mpsc::channel(BUFFER);
        let _ = tokio::spawn(async move { get_urls(receiver).await });
        let validator = Validator {
            schemas: Arc::new(RwLock::new(schemas(&validation_options))),
            validation_options,
            cache,
            urls,
            sender,
        };
        validator
    }

    /// Validates a single value.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_validate::Validator;
    /// use stac::Item;
    ///
    /// let item = Item::new("an-id");
    /// # tokio_test::block_on(async {
    /// let validator = Validator::new().await;
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
            Err(Error::CannotValidate(value))
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
            let r#type: Type = object
                .get("type")
                .and_then(|v| v.as_str())
                .map(|t| t.parse::<Type>())
                .transpose()?
                .ok_or(Error::NoType)?;
            if r#type == Type::ItemCollection {
                if let Some(features) = object.get("features") {
                    return validator.validate(features).await;
                } else {
                    return Ok(());
                }
            }
            let version: Version = object
                .get("stac_version")
                .and_then(|v| v.as_str())
                .map(|v| v.parse::<Version>())
                .transpose()
                .unwrap()
                .ok_or(Error::NoVersion)?;

            let url = build_schema_url(r#type, &version);
            let schema = validator.schema(url).await?;
            let value = Arc::new(Value::Object(object));
            let mut result = schema
                .validate(&value)
                .map_err(Error::from_validation_errors);
            if result.is_ok() {
                result = validator.validate_extensions(value.clone()).await
            }
            if let Err(err) = result {
                let mut join_set = JoinSet::new();
                {
                    let mut urls = validator.urls.lock().unwrap();
                    if urls.is_empty() {
                        return Err(err);
                    } else {
                        for url in urls.drain() {
                            let validator = validator.clone();
                            let _ = join_set.spawn(async move { validator.resolve(url).await });
                        }
                    }
                }
                while let Some(result) = join_set.join_next().await {
                    let _ = result??;
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
                let extension = Url::parse(extension)?;
                let validator = self.clone();
                let value = value.clone();
                let _ = join_set.spawn(async move {
                    let extension_schema = validator.schema(extension).await?;
                    extension_schema
                        .validate(&value)
                        .map_err(Error::from_validation_errors)
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

    async fn schema(&self, url: Url) -> Result<Arc<JsonschemaValidator>> {
        {
            let schemas = self.schemas.read().await;
            if let Some(schema) = schemas.get(&url) {
                return Ok(schema.clone());
            }
        }
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.sender.send((url.clone(), sender)).await?;
        let value = receiver.await??;
        let schema = self
            .validation_options
            .build(&value)
            .map_err(|err| Error::from_validation_errors([err].into_iter()))?;
        let schema = Arc::new(schema);
        {
            let mut schemas = self.schemas.write().await;
            let _ = schemas.insert(url, schema.clone());
        }
        Ok(schema)
    }

    async fn resolve(&self, url: Url) -> Result<()> {
        let (sender, receiver) = tokio::sync::oneshot::channel();
        self.sender.send((url.clone(), sender)).await?;
        let value = receiver.await??;
        {
            let mut cache = self.cache.write().unwrap();
            let _ = cache.insert(url, value);
        }
        Ok(())
    }
}

impl SchemaResolver for Resolver {
    fn resolve(
        &self,
        _: &Value,
        url: &Url,
        _: &str,
    ) -> std::result::Result<Arc<Value>, SchemaResolverError> {
        {
            let cache = self.cache.read().unwrap();
            if let Some(schema) = cache.get(url) {
                return Ok(schema.clone());
            }
        }
        {
            let mut urls = self.urls.lock().unwrap();
            let _ = urls.insert(url.clone());
        }
        Err(SchemaResolverError::msg("need to resolve and re-try"))
    }
}

fn build_schema_url(r#type: Type, version: &Version) -> Url {
    Url::parse(&format!(
        "{}{}",
        SCHEMA_BASE,
        r#type
            .spec_path(version)
            .expect("we shouldn't get here with an item collection")
    ))
    .unwrap()
}

fn schemas(validation_options: &ValidationOptions) -> HashMap<Url, Arc<JsonschemaValidator>> {
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

    schema!(Item, v1_0_0, "../schemas/v1.0.0/item.json", schemas);
    schema!(Catalog, v1_0_0, "../schemas/v1.0.0/catalog.json", schemas);
    schema!(
        Collection,
        v1_0_0,
        "../schemas/v1.0.0/collection.json",
        schemas
    );
    schema!(Item, v1_1_0, "../schemas/v1.1.0/item.json", schemas);
    schema!(Catalog, v1_1_0, "../schemas/v1.1.0/catalog.json", schemas);
    schema!(
        Collection,
        v1_1_0,
        "../schemas/v1.1.0/collection.json",
        schemas
    );

    schemas
}

fn cache() -> HashMap<Url, Arc<Value>> {
    let mut cache = HashMap::new();

    macro_rules! resolve {
        ($url:expr, $path:expr) => {
            let _ = cache.insert(
                Url::parse($url).unwrap(),
                Arc::new(serde_json::from_str(include_str!($path)).unwrap()),
            );
        };
    }

    // General
    resolve!(
        "https://geojson.org/schema/Feature.json",
        "../schemas/geojson/Feature.json"
    );
    resolve!(
        "https://geojson.org/schema/Geometry.json",
        "../schemas/geojson/Geometry.json"
    );
    resolve!(
        "http://json-schema.org/draft-07/schema",
        "../schemas/json-schema/draft-07.json"
    );

    // STAC v1.0.0
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/basics.json",
        "../schemas/v1.0.0/basics.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/datetime.json",
        "../schemas/v1.0.0/datetime.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/instrument.json",
        "../schemas/v1.0.0/instrument.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/item.json",
        "../schemas/v1.0.0/item.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/licensing.json",
        "../schemas/v1.0.0/licensing.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.0.0/item-spec/json-schema/provider.json",
        "../schemas/v1.0.0/provider.json"
    );

    // STAC v1.1.0
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/bands.json",
        "../schemas/v1.1.0/bands.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/basics.json",
        "../schemas/v1.1.0/basics.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/common.json",
        "../schemas/v1.1.0/common.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/data-values.json",
        "../schemas/v1.1.0/data-values.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/datetime.json",
        "../schemas/v1.1.0/datetime.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/instrument.json",
        "../schemas/v1.1.0/instrument.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/item.json",
        "../schemas/v1.1.0/item.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/licensing.json",
        "../schemas/v1.1.0/licensing.json"
    );
    resolve!(
        "https://schemas.stacspec.org/v1.1.0/item-spec/json-schema/provider.json",
        "../schemas/v1.1.0/provider.json"
    );

    cache
}

async fn get_urls(mut receiver: Receiver<(Url, OneshotSender<Result<Arc<Value>>>)>) -> Result<()> {
    let mut cache: HashMap<Url, Arc<Value>> = HashMap::new();
    let mut gets: HashMap<Url, Vec<OneshotSender<Result<Arc<Value>>>>> = HashMap::new();
    let client = Client::new();
    let (local_sender, mut local_receiver) = tokio::sync::mpsc::channel(BUFFER);
    loop {
        match receiver.try_recv() {
            Err(TryRecvError::Disconnected) => return Ok(()),
            Err(TryRecvError::Empty) => match local_receiver.try_recv() {
                Err(TryRecvError::Disconnected) => return Ok(()),
                Err(TryRecvError::Empty) => tokio::task::yield_now().await,
                Ok((url, result)) => {
                    let mut senders = gets
                        .remove(&url)
                        .expect("all sent values should be in gets");
                    match result {
                        Ok(value) => {
                            let value = Arc::<Value>::new(value);
                            let _ = cache.insert(url, value.clone());
                            for sender in senders {
                                sender.send(Ok(value.clone())).unwrap();
                            }
                        }
                        Err(err) => {
                            senders
                                .pop()
                                .expect("there should be at least one sender")
                                .send(Err(err))
                                .unwrap();
                        }
                    };
                }
            },
            Ok((url, sender)) => {
                if let Some(value) = cache.get(&url) {
                    sender.send(Ok(value.clone())).unwrap();
                } else {
                    gets.entry(url.clone())
                        .or_insert_with(|| {
                            tracing::debug!("getting url: {}", url);
                            let local_sender = local_sender.clone();
                            let client = client.clone();
                            let _ = tokio::spawn(async move {
                                match get(client, url.clone()).await {
                                    Ok(value) => local_sender.send((url, Ok(value))).await,
                                    Err(err) => local_sender.send((url, Err(err))).await,
                                }
                            });
                            Vec::new()
                        })
                        .push(sender);
                }
            }
        }
    }
}

async fn get(client: Client, url: Url) -> Result<Value> {
    client
        .get(url)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await
        .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use super::Validator;
    use stac::Item;

    #[tokio::test]
    async fn validate_array() {
        let items: Vec<_> = (0..100)
            .map(|i| Item::new(format!("item-{}", i)))
            .map(|i| serde_json::to_value(i).unwrap())
            .collect();
        let validator = Validator::new().await;
        validator.validate(&items).await.unwrap();
    }
}
