use std::{collections::HashMap, sync::Arc};

use crate::{Error, Result};

use object_store::{local::LocalFileSystem, path::Path, DynObjectStore, ObjectStoreScheme};
use once_cell::sync::Lazy;
use tokio::sync::RwLock;
use url::Url;

// To avoid memory leaks, we clear the cache when it grows too big.
// The value does not have any meaning, other than polars use the same.
const CACHE_SIZE: usize = 8;

static OBJECT_STORE_CACHE: Lazy<RwLock<HashMap<ObjectStoreIdentifier, Arc<DynObjectStore>>>> =
    Lazy::new(Default::default);

/// Parameter set to identify and cache an object store
#[derive(PartialEq, Eq, Hash, Debug)]
struct ObjectStoreIdentifier {
    /// A base url to the bucket.
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

#[cfg(any(
    feature = "object-store-aws",
    feature = "object-store-gcp",
    feature = "object-store-azure"
))]
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

/// This was yanked from [object_store::parse_url_opts] with the following changes:
///
/// - Build [object_store::ObjectStore] with environment variables
/// - Return [Arc] instead of [Box]
#[cfg_attr(
    not(any(
        feature = "object-store-aws",
        feature = "object-store-gcp",
        feature = "object-store-azure",
        feature = "object-store-http"
    )),
    allow(unused_variables)
)]
fn create_object_store<I, K, V>(
    scheme: ObjectStoreScheme,
    url: &Url,
    options: I,
) -> Result<Arc<DynObjectStore>>
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
        s => return Err(Error::ObjectStoreCreate { scheme: s }),
    };
    Ok(store)
}

/// Drop-in replacement for [object_store::parse_url_opts] with caching and env vars.
///
/// It will create or retrieve object store based on passed `url` and `options`.
/// Keeps global cache
pub async fn parse_url_opts<I, K, V>(
    url: &Url,
    options: I,
) -> crate::Result<(Arc<DynObjectStore>, Path)>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    let (scheme, path) = ObjectStoreScheme::parse(url).map_err(object_store::Error::from)?;

    let base_url = url
        .as_ref()
        .strip_suffix(path.as_ref())
        .unwrap_or_default()
        .try_into()?;

    let object_store_id = ObjectStoreIdentifier::new(base_url, options);
    let options = object_store_id.get_options();

    {
        let cache = OBJECT_STORE_CACHE.read().await;
        if let Some(store) = (*cache).get(&object_store_id) {
            return Ok((store.clone(), path));
        }
    }
    let store = create_object_store(scheme, url, options)?;
    {
        let mut cache = OBJECT_STORE_CACHE.write().await;

        if cache.len() >= CACHE_SIZE {
            (*cache).clear()
        }
        _ = (*cache).insert(object_store_id, store.clone());
    }

    Ok((store.clone(), path))
}

#[cfg(test)]
mod tests {
    use url::Url;

    use super::*;

    #[tokio::test]
    async fn file_different_path() {
        let options: Vec<(String, String)> = Vec::new();

        let url = Url::parse("file:///some/path").unwrap();
        let (store, path) = parse_url_opts(&url, options.clone()).await.unwrap();

        let url2 = Url::parse("file:///other/path").unwrap();
        let (store2, _) = parse_url_opts(&url2, options.clone()).await.unwrap();

        {
            let cache = OBJECT_STORE_CACHE.read().await;
            println!("{cache:#?}")
        }

        assert!(Arc::ptr_eq(&store, &store2));
        assert!(std::ptr::addr_eq(Arc::as_ptr(&store), Arc::as_ptr(&store2)));
        // assert_eq!(store.as_ref(), store2.as_ref());
        // assert_eq!(Arc::as_ptr(&store), Arc::as_ptr(&store2));
        assert_eq!(path.as_ref(), "some/path");
    }

    #[tokio::test]
    async fn file_different_options() {
        let options: Vec<(String, String)> = Vec::new();

        let url = Url::parse("file:///some/path").unwrap();
        let (store, _) = parse_url_opts(&url, options).await.unwrap();

        let options2: Vec<(String, String)> = vec![(String::from("some"), String::from("option"))];
        let url2 = Url::parse("file:///some/path").unwrap();
        let (store2, _) = parse_url_opts(&url2, options2).await.unwrap();

        assert!(!Arc::ptr_eq(&store, &store2));
    }

    #[cfg(feature = "object-store-aws")]
    #[tokio::test]
    async fn cache_works() {
        let url = Url::parse("s3://bucket/item").unwrap();
        let options: Vec<(String, String)> = Vec::new();

        let (store1, path) = parse_url_opts(&url, options.clone()).await.unwrap();

        let url2 = Url::parse("s3://bucket/item2").unwrap();
        let (store2, _path) = parse_url_opts(&url2, options.clone()).await.unwrap();

        assert!(Arc::ptr_eq(&store1, &store2));
        assert_eq!(path.as_ref(), "item");
    }

    #[cfg(feature = "object-store-aws")]
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

    #[cfg(feature = "object-store-aws")]
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
