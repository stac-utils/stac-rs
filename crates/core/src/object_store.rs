use object_store::{
    local::LocalFileSystem, memory::InMemory, path::Path, DynObjectStore, ObjectStoreScheme,
};
use url::Url;

#[cfg(feature = "object-store")]
macro_rules! builder_env_opts {
    ($builder:ty, $url:expr, $options:expr) => {{
        let builder = $options.into_iter().fold(
            <$builder>::from_env().with_url($url.to_string()),
            |builder, (key, value)| match key.as_ref().parse() {
                Ok(k) => builder.with_config(k, value),
                Err(_) => builder,
            },
        );
        Box::new(builder.build()?)
    }};
}

/// Modified version of object_store::parse_url_opts that also parses env
///
/// It does the same, except we start from env vars, then apply url and then overrides from options
///
/// This is POC. To improve on this idea, maybe it's good to "cache" a box with dynamic ObjectStore for each bucket we access, since ObjectStore have some logic inside tied to a bucket level, like connection pooling, credential caching
pub fn parse_url_opts<I, K, V>(
    url: &Url,
    options: I,
) -> Result<(Box<DynObjectStore>, Path), object_store::Error>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    let _options = options;
    let (scheme, path) = ObjectStoreScheme::parse(url)?;
    let path = Path::parse(path)?;

    let store: Box<DynObjectStore> = match scheme {
        ObjectStoreScheme::Local => Box::new(LocalFileSystem::new()),
        ObjectStoreScheme::Memory => Box::new(InMemory::new()),
        #[cfg(feature = "object-store-aws")]
        ObjectStoreScheme::AmazonS3 => {
            builder_env_opts!(object_store::aws::AmazonS3Builder, url, _options)
        }
        #[cfg(feature = "object-store-gcp")]
        ObjectStoreScheme::GoogleCloudStorage => {
            builder_env_opts!(object_store::gcp::GoogleCloudStorageBuilder, url, _options)
        }
        #[cfg(feature = "object-store-azure")]
        ObjectStoreScheme::MicrosoftAzure => {
            builder_env_opts!(object_store::azure::MicrosoftAzureBuilder, url, _options)
        }
        #[cfg(feature = "object-store-http")]
        ObjectStoreScheme::Http => {
            let url = &url[..url::Position::BeforePath];
            let builder = _options.into_iter().fold(
                object_store::http::HttpBuilder::new().with_url(url.to_string()),
                |builder, (key, value)| match key.as_ref().parse() {
                    Ok(k) => builder.with_config(k, value),
                    Err(_) => builder,
                },
            );
            Box::new(builder.build()?)
        }
        s => {
            return Err(object_store::Error::Generic {
                store: "parse_url",
                source: format!("feature for {s:?} not enabled").into(),
            })
        }
    };
    Ok((store, path))
}
