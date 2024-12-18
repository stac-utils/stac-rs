use std::{collections::HashMap, sync::Arc};

use object_store::{local::LocalFileSystem, path::Path, DynObjectStore, ObjectStoreScheme};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use url::Url;

static OBJECT_STORE_CACHE: Lazy<RwLock<HashMap<ObjectStoreIdentifier, Arc<DynObjectStore>>>> =
    Lazy::new(Default::default);

/// Parameter set to identify and cache an object Storage
#[derive(PartialEq, Eq, Hash)]
struct ObjectStoreIdentifier {
    /// A base url to the bucket.
    // should be enough to identify cloud provider and bucket
    base_url: Url,

    /// Object Store options
    options: Vec<(String, String)>,
}

impl ObjectStoreIdentifier {
    fn new<I, K, V>(base_url: Url, options: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: AsRef<str>,
        V: Into<String>,
    {
        Self {
            base_url,
            options: options
                .into_iter()
                .map(|(k, v)| (k.as_ref().into(), v.into()))
                .collect(),
        }
    }

    fn get_options(&self) -> Vec<(String, String)> {
        self.options.to_owned()
    }
}

macro_rules! builder_env_opts {
    ($builder:ty, $url:expr, $options:expr) => {{
        let builder = $options.into_iter().fold(
            <$builder>::from_env().with_url($url.to_string()),
            |builder, (key, value)| match key.as_ref().parse() {
                Ok(k) => builder.with_config(k, value),
                Err(_) => builder,
            },
        );
        Arc::new(builder.build()?)
    }};
}

fn create_object_store<I, K, V>(
    scheme: ObjectStoreScheme,
    url: &Url,
    options: I,
) -> Result<Arc<DynObjectStore>, object_store::Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    let store: Arc<DynObjectStore> = match scheme {
        ObjectStoreScheme::Local => Arc::new(LocalFileSystem::new()),
        #[cfg(feature = "object-store-aws")]
        ObjectStoreScheme::AmazonS3 => {
            builder_env_opts!(object_store::aws::AmazonS3Builder, url, options)
        }
        #[cfg(feature = "object-store-gcp")]
        ObjectStoreScheme::GoogleCloudStorage => {
            builder_env_opts!(object_store::gcp::GoogleCloudStorageBuilder, url, options)
        }
        #[cfg(feature = "object-store-azure")]
        ObjectStoreScheme::MicrosoftAzure => {
            builder_env_opts!(object_store::azure::MicrosoftAzureBuilder, url, options)
        }
        #[cfg(feature = "object-store-http")]
        ObjectStoreScheme::Http => {
            let url = &url[..url::Position::BeforePath];
            let builder = options.into_iter().fold(
                object_store::http::HttpBuilder::new().with_url(url.to_string()),
                |builder, (key, value)| match key.as_ref().parse() {
                    Ok(k) => builder.with_config(k, value),
                    Err(_) => builder,
                },
            );
            Arc::new(builder.build()?)
        }
        s => {
            return Err(object_store::Error::Generic {
                store: "parse_url",
                source: format!("feature for {s:?} not enabled").into(),
            })
        }
    };
    Ok(store)
}

/// Modified version of object_store::parse_url_opts that also parses env
///
/// It does the same, except we start from env vars, then apply url and then overrides from options
///
/// This is POC. To improve on this idea, maybe it's good to "cache" a box with dynamic ObjectStore for each bucket we access, since ObjectStore have some logic inside tied to a bucket level, like connection pooling, credential caching
pub async fn parse_url_opts<I, K, V>(
    url: &Url,
    options: I,
) -> Result<(Arc<DynObjectStore>, Path), crate::Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    // TODO: Handle error properly
    let (scheme, path) = ObjectStoreScheme::parse(url).unwrap();

    let path_string: String = path.clone().into();
    let path_str = path_string.as_str();
    // TODO: Handle error properly
    let base_url = url[..]
        .strip_suffix(path_str)
        .unwrap_or_default()
        .try_into()
        .unwrap();

    let object_store_id = ObjectStoreIdentifier::new(base_url, options);
    let options = object_store_id.get_options();

    {
        let cache = OBJECT_STORE_CACHE.read().await;
        if let Some(store) = cache.get(&object_store_id) {
            return Ok((store.clone(), path));
        }
    }

    let store = create_object_store(scheme, url, options)?;
    {
        let mut cache = OBJECT_STORE_CACHE.write().await;

        // TODO: Do we need this cache clean? What is a reasonable cache size here?
        if cache.len() >= 8 {
            cache.clear()
        }
        _ = cache.insert(object_store_id, store.clone());
    }

    Ok((store.clone(), path))
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[tokio::test]
    async fn cache_works() {
        let url = Url::parse("s3://bucket/item").unwrap();
        let options: Vec<(String, String)> = Vec::new();

        let (store1, _path) = parse_url_opts(&url, options.clone()).await.unwrap();

        let url2 = Url::parse("s3://bucket/item2").unwrap();
        let (store2, _path) = parse_url_opts(&url2, options.clone()).await.unwrap();

        assert!(Arc::ptr_eq(&store1, &store2));
    }
    #[tokio::test]
    async fn different_options() {
        let url = Url::parse("s3://bucket/item").unwrap();
        let options: Vec<(String, String)> = Vec::new();

        let (store, _path) = parse_url_opts(&url, options).await.unwrap();

        let url2 = Url::parse("s3://bucket/item2").unwrap();
        let options2: Vec<(String, String)> = vec![(String::from("some"), String::from("option"))];
        let (store2, _path) = parse_url_opts(&url2, options2).await.unwrap();

        assert!(!Arc::ptr_eq(&store, &store2));
    }
    #[tokio::test]
    async fn different_urls() {
        let url = Url::parse("s3://bucket/item").unwrap();
        let options: Vec<(String, String)> = Vec::new();

        let (store, _path) = parse_url_opts(&url, options.clone()).await.unwrap();

        let url2 = Url::parse("s3://other-bucket/item").unwrap();
        // let options2: Vec<(String, String)> = vec![(String::from("some"), String::from("option"))];
        let (store2, _path) = parse_url_opts(&url2, options).await.unwrap();

        assert!(!Arc::ptr_eq(&store, &store2));
    }
}
