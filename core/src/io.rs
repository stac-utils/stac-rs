//! Input and output (IO) functions.

use crate::{Error, Format, Href, Result};
#[cfg(feature = "reqwest")]
use reqwest::blocking::Response;
use serde::de::DeserializeOwned;
use std::{fs::File, path::Path};
use url::Url;
#[cfg(feature = "object-store")]
use {
    crate::object_store::Get,
    object_store::{
        buffered::BufWriter, local::LocalFileSystem, path::Path as ObjectStorePath, ObjectStore,
    },
    std::sync::Arc,
};

/// Reads any STAC value from an href.
///
/// If the `geoparquet` feature is enabled, and the href's extension is
/// `geoparquet` or `parquet`, the data will be read as
/// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet). This is
/// more inefficient than using [crate::geoparquet::read], so prefer that if you
/// know your href points to geoparquet data.
///
/// Use [crate::json::read] if you want to ensure that your data are read as JSON.
///
/// # Examples
///
/// ```
/// let item: stac::Item = stac::read("examples/simple-item.json").unwrap();
/// ```
pub fn read<T: Href + DeserializeOwned>(href: impl ToString) -> Result<T> {
    Config::new().read(href)
}

/// Gets any STAC value from an href.
///
/// This is an asynchronous function that uses
/// [object_store](https://docs.rs/object_store/latest/object_store/) via the
/// `object-store` feature. The `object_store` feature only includes local
/// filesystem storage, so to read from http endpoints or a cloud store, you
/// need to enable the corresponding feature (`object-store-http`,
/// `object-store-aws`, `object-store-gcp`, or `object-store-azure`).
///
/// For more control over access, e.g. to pass options to the underlying
/// **object_store**, use [Config].
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// #[cfg(feature = "object-store")]
/// {
///     let item: stac::Item = stac::get("examples/simple-item.json").await.unwrap();
/// }
/// # })
/// ```
#[cfg(feature = "object-store")]
pub async fn get<T: Href + DeserializeOwned>(href: impl ToString) -> Result<T> {
    Config::new().get(href.to_string()).await
}

/// Configuration for reading and writing STAC values.
#[derive(Debug, Default, Clone)]
pub struct Config {
    format: Option<Format>,
    options: Vec<KeyValue>,
}

/// A key and a value.
pub type KeyValue = (String, String);

impl Config {
    /// Creates a new reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::io::Config;
    ///
    /// let config = Config::new();
    /// ```
    pub fn new() -> Config {
        Config {
            format: None,
            options: Vec::new(),
        }
    }

    /// Sets the format for this Config.
    ///
    /// If the format is not set, it will be inferred from the href's extension,
    /// falling back to JSON.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{io::Config, Format};
    ///
    /// let config = Config::new().format(Format::NdJson);
    /// ```
    pub fn format(mut self, format: impl Into<Option<Format>>) -> Config {
        self.format = format.into();
        self
    }

    /// Sets the options for this config.
    ///
    /// The options are used to configure the object store, if one is used.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::io::Config;
    ///
    /// let config = Config::new().options(vec![("foo".to_string(), "bar".to_string())]);
    /// ```
    pub fn options(mut self, options: impl Into<Vec<KeyValue>>) -> Config {
        self.options = options.into();
        self
    }

    /// Reads a STAC value from an href.
    ///
    /// This is a synchronous operation that can only read from the local
    /// filesystem or, if the `reqwest` feature is enabled, from `http` and
    /// `https` hrefs. If you need support for cloud storage (e.g. aws, azure,
    /// or gcp), use the asynchronous [Config::get].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{io::Config, Item};
    ///
    /// let config = Config::new();
    /// let item: Item = config.read("examples/simple-item.json").unwrap();
    /// ```
    pub fn read<T>(&self, href: impl ToString) -> Result<T>
    where
        T: DeserializeOwned + Href,
    {
        let href = href.to_string();
        match self
            .format
            .unwrap_or_else(|| Format::infer_from_href(&href).unwrap_or_default())
        {
            Format::Json(_) => crate::json::read(href),
            Format::NdJson => {
                serde_json::from_value(serde_json::to_value(crate::ndjson::read(href)?)?)
                    .map_err(Error::from)
            }
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => {
                serde_json::from_value(serde_json::to_value(crate::geoparquet::read(href)?)?)
                    .map_err(Error::from)
            }
        }
    }

    /// Reads a STAC value from a [Read](std::io::Read).
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{io::Config, Item};
    /// use std::fs::File;
    ///
    /// let config = Config::new();
    /// let file = File::open("examples/simple-item.json").unwrap();
    /// let item: Item = config.from_reader(file).unwrap();
    /// ```
    #[cfg_attr(not(feature = "geoparquet"), allow(unused_mut))]
    pub fn from_reader<T>(&self, mut read: impl std::io::Read) -> Result<T>
    where
        T: DeserializeOwned + Href,
    {
        match self.format.unwrap_or_default() {
            Format::Json(_) => serde_json::from_reader(read).map_err(Error::from),
            Format::NdJson => serde_json::from_value(serde_json::to_value(
                crate::ndjson::from_buf_reader(std::io::BufReader::new(read))?,
            )?)
            .map_err(Error::from),
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => {
                let mut buf = Vec::new();
                let _ = read.read_to_end(&mut buf)?;
                serde_json::from_value(serde_json::to_value(crate::geoparquet::from_reader(
                    bytes::Bytes::from(buf),
                )?)?)
                .map_err(Error::from)
            }
        }
    }

    /// Gets a STAC value from an href using **object_store**.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{io::Config, Item};
    ///
    /// let config = Config::new();
    /// # tokio_test::block_on(async {
    /// let item: Item = config.get("examples/simple-item.json").await.unwrap();
    /// # })
    /// ```
    #[cfg(feature = "object-store")]
    pub async fn get<T: Href + DeserializeOwned>(&self, href: impl ToString) -> Result<T> {
        use send_future::SendFuture as _;

        // TODO make a `get_opts` to allow use to pass `GetOptions` in
        let href = href.to_string();
        let (object_store, path) = self.parse_href(&href)?;
        let mut value: T = match self
            .format
            .unwrap_or_else(|| Format::infer_from_href(&href).unwrap_or_default())
        {
            Format::Json(_) => object_store.get_json(&path).send().await?,
            Format::NdJson => serde_json::from_value(serde_json::to_value(
                object_store.get_ndjson(&path).send().await?,
            )?)?,
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => serde_json::from_value(serde_json::to_value(
                object_store.get_geoparquet(&path).send().await?,
            )?)?,
        };
        value.set_href(href);
        Ok(value)
    }

    /// Gets an [ItemCollection](crate::ItemCollection) from an href using **object_store**.
    ///
    /// Use this method when you know you're getting an item collection, e.g. if
    /// you're reading
    /// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet).
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::io::Config;
    ///
    /// let config = Config::new();
    /// # tokio_test::block_on(async {
    /// #[cfg(feature = "geoparquet")] {
    ///     let item_collection = config.get_item_collection("data/extended-item.parquet").await.unwrap();
    /// }
    /// # })
    /// ```
    #[cfg(feature = "object-store")]
    pub async fn get_item_collection(&self, href: impl ToString) -> Result<crate::ItemCollection> {
        let href = href.to_string();
        let (object_store, path) = self.parse_href(&href)?;
        let mut item_collection = match self
            .format
            .unwrap_or_else(|| Format::infer_from_href(&href).unwrap_or_default())
        {
            Format::Json(_) => object_store.get_json(&path).await?,
            Format::NdJson => object_store.get_ndjson(&path).await?,
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => object_store.get_geoparquet(&path).await?,
        };
        item_collection.set_href(href);
        Ok(item_collection)
    }

    /// Creates an [object_store::buffered::BufWriter] for the provided url.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::io::Config;
    ///
    /// let config = Config::new();
    /// let buf_writer = config.buf_writer(&"s3://stac/item.json".parse().unwrap()).unwrap();
    /// ```
    #[cfg(feature = "object-store")]
    pub fn buf_writer(&self, url: &Url) -> Result<BufWriter> {
        let (object_store, path) = object_store::parse_url_opts(url, self.iter_options())?;
        Ok(BufWriter::new(Arc::new(object_store), path))
    }

    /// Creates an [object_store::ObjectStore] and path for the provided url.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::io::Config;
    ///
    /// let config = Config::new();
    /// let (store, path) = config.object_store(&"s3://stac/item.json".parse().unwrap()).unwrap();
    /// ```
    #[cfg(feature = "object-store")]
    pub fn object_store(&self, url: &Url) -> Result<(Box<dyn ObjectStore>, ObjectStorePath)> {
        object_store::parse_url_opts(url, self.iter_options()).map_err(Error::from)
    }

    #[cfg(feature = "object-store")]
    fn parse_href(&self, href: &str) -> Result<(Box<dyn ObjectStore>, ObjectStorePath)> {
        if let Ok(url) = Url::parse(href) {
            object_store::parse_url_opts(&url, self.iter_options()).map_err(Error::from)
        } else {
            let path = ObjectStorePath::from_filesystem_path(href)?;
            Ok((Box::new(LocalFileSystem::new()), path))
        }
    }

    #[cfg(feature = "object-store")]
    fn iter_options(&self) -> impl Iterator<Item = (&str, &str)> {
        self.options.iter().map(|(k, v)| (k.as_str(), v.as_str()))
    }
}

pub(crate) trait Read<T: Href + DeserializeOwned> {
    fn read(href: impl ToString) -> Result<T> {
        let href = href.to_string();
        let mut value: T = if let Some(url) = crate::href_to_url(&href) {
            Self::read_from_url(url)?
        } else {
            Self::read_from_path(&href)?
        };
        value.set_href(href);
        Ok(value)
    }

    fn read_from_path(path: impl AsRef<Path>) -> Result<T> {
        let file = File::open(path.as_ref())?;
        Self::read_from_file(file)
    }

    fn read_from_file(file: File) -> Result<T>;

    #[cfg(feature = "reqwest")]
    fn read_from_url(url: Url) -> Result<T> {
        let response = reqwest::blocking::get(url.clone())?;
        Self::from_response(response)
    }

    #[cfg(feature = "reqwest")]
    fn from_response(response: Response) -> Result<T>;

    #[cfg(not(feature = "reqwest"))]
    fn read_from_url(_: Url) -> Result<T> {
        Err(Error::ReqwestNotEnabled)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Catalog, Collection, Item, ItemCollection};

    macro_rules! read {
        ($function:ident, $filename:expr, $value:ty $(, $meta:meta)?) => {
            #[test]
            $(#[$meta])?
            fn $function() {
                use crate::Href;

                let value: $value = crate::read($filename).unwrap();
                assert!(value.href().is_some());
            }
        };
    }

    read!(read_item_from_path, "examples/simple-item.json", Item);
    read!(read_catalog_from_path, "examples/catalog.json", Catalog);
    read!(
        read_collection_from_path,
        "examples/collection.json",
        Collection
    );
    read!(
        read_item_collection_from_path,
        "data/item-collection.json",
        ItemCollection
    );

    #[cfg(feature = "reqwest")]
    mod with_reqwest {
        use crate::{Catalog, Collection, Item};

        read!(
            read_item_from_url,
            "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json",
            Item
        );
        read!(
            read_catalog_from_url,
            "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/catalog.json",
            Catalog
        );
        read!(
            read_collection_from_url,
            "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/collection.json",
            Collection
        );
    }

    #[cfg(not(feature = "reqwest"))]
    mod without_reqwest {
        #[test]
        fn read_url() {
            assert!(matches!(
                crate::read::<crate::Item>("http://stac-rs.test/item.json").unwrap_err(),
                crate::Error::ReqwestNotEnabled
            ));
        }
    }

    #[test]
    #[cfg(feature = "geoparquet")]
    fn read_geoparquet() {
        let _: ItemCollection = super::read("data/extended-item.parquet").unwrap();
    }

    #[test]
    #[cfg(not(feature = "geoparquet"))]
    fn read_geoparquet() {
        let _ = super::read::<ItemCollection>("data/extended-item.parquet").unwrap_err();
    }

    #[tokio::test]
    #[cfg(feature = "object-store")]
    async fn get() {
        use crate::Href;

        let item: Item = super::get("examples/simple-item.json").await.unwrap();
        assert!(item.href().unwrap().ends_with("examples/simple-item.json"));
    }
}
