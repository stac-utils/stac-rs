//! Read and write STAC.
//!
//! # Reading
//!
//! ```
//! let item: stac::Item = stac::read("examples/simple-item.json").unwrap();
//! ```
//!
//! The format of the data are inferred from the href extension, e.g. if the
//! `geoparquet` feature is enabled, `*.parquet` and `*.geoparquet` files will
//! be read as such:
//!
//! ```
//! use stac::ItemCollection;
//!
//! #[cfg(feature = "geoparquet")]
//! {
//!     let item_collection: ItemCollection = stac::read("data/extended-item.parquet").unwrap();
//! }
//! ```
//!
//! To specify the format, use [Format::read].
//!
//! ## Object store
//!
//! If the `object-store` feature (and one of its sub-features, e.g. `object-store-aws`) is enabled, you can get values from cloud storage:
//!
//! ```no_run
//! use stac::Item;
//!
//! #[cfg(feature = "object-store-aws")]
//! {
//! # tokio_test::block_on(async {
//!     let item: Item = stac::io::get("s3://bucket/item.json").await.unwrap();
//! # });
//! }
//! ```
//!
//! To provide options, e.g. to configure your AWS credentials, use [get_opts], which will forward the options to [object_store::parse_url_opts]:
//!
//! ```no_run
//! # use stac::Item;
//! # #[cfg(feature = "object-store-aws")]
//! # {
//! # tokio_test::block_on(async {
//! let item: Item = stac::io::get_opts("s3://bucket/item.json", [("aws_access_key_id", "...")]).await.unwrap();
//! # });
//! # }
//! ```
//!
//! # Writing
//!
//! ```no_run
//! use stac::{Item, Format};
//!
//! let item = Item::new("an-id");
//! stac::write("an-id.json", item).unwrap();
//! ```
//!
//! ## Object store
//!
//! [put] and [put_opts] write objects to an object store:
//!
//! ```no_run
//! use stac::Item;
//! let item = Item::new("an-id");
//! #[cfg(feature = "object-store-aws")]
//! {
//! # tokio_test::block_on(async {
//!     stac::io::put_opts(
//!         "s3://bucket/item.json",
//!         item,
//!         [("aws_access_key_id", "...")],
//!     ).await.unwrap();
//! # });
//! }
//! ```

use crate::{
    geoparquet::{FromGeoparquet, IntoGeoparquet},
    json::{FromJson, ToJson},
    ndjson::{FromNdjson, ToNdjson},
    Format, Href, Result, SelfHref,
};
use std::path::Path;

/// Reads a STAC value from an href.
///
/// The format will be inferred from the href's extension. If you want to
/// specify the format, use [Format::read].
///
/// # Examples
///
/// ```
/// let item: stac::Item = stac::read("examples/simple-item.json").unwrap();
/// ```
pub fn read<T: SelfHref + FromJson + FromNdjson + FromGeoparquet>(
    href: impl Into<Href>,
) -> Result<T> {
    let href = href.into();
    let format = Format::infer_from_href(href.as_str()).unwrap_or_default();
    format.read(href)
}

/// Gets a value, maybe from an object store.
///
/// # Examples
///
/// ```no_run
/// use stac::Item;
///
/// #[cfg(feature = "object-store-aws")]
/// {
/// # tokio_test::block_on(async {
///     let item: Item = stac::io::get("s3://bucket/item.json").await.unwrap();
/// # })
/// }
/// ```
#[cfg(feature = "object-store")]
pub async fn get<T: SelfHref + FromJson + FromNdjson + FromGeoparquet>(
    href: impl Into<Href>,
) -> Result<T> {
    let options: [(&str, &str); 0] = [];
    get_opts(href, options).await
}

/// Gets a value, maybe from an object store with the provided options.
///
/// If `href` is a url, [object_store::parse_url_opts] will be used to build the object store to get the value.
///
/// # Examples
///
/// ```no_run
/// use stac::Item;
///
/// #[cfg(feature = "object-store-aws")]
/// {
/// # tokio_test::block_on(async {
///     let item: Item = stac::io::get_opts("s3://bucket/item.json", [("aws_access_key_id", "...")]).await.unwrap();
/// # })
/// }
/// ```
#[cfg(feature = "object-store")]
pub async fn get_opts<T, I, K, V>(href: impl Into<Href>, options: I) -> Result<T>
where
    T: SelfHref + FromJson + FromNdjson + FromGeoparquet,
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    let href = href.into();
    let format = Format::infer_from_href(href.as_str()).unwrap_or_default();
    format.get_opts(href, options).await
}

/// Writes a STAC value to a path.
///
/// The format will be inferred from the href's extension. If you want to
/// specify the format, use [Format::write].
///
/// # Examples
///
/// ```no_run
/// use stac::Item;
///
/// let item = Item::new("an-id");
/// stac::write("an-id.json", item).unwrap();
/// ```
pub fn write<T: ToJson + ToNdjson + IntoGeoparquet>(
    path: impl AsRef<Path>,
    value: T,
) -> Result<()> {
    let path = path.as_ref();
    let format = path
        .to_str()
        .and_then(Format::infer_from_href)
        .unwrap_or_default();
    format.write(path, value)
}

/// Puts a value, maybe to an object store.
///
/// # Examples
///
/// ```no_run
/// use stac::Item;
///
/// #[cfg(feature = "object-store-aws")]
/// {
/// let item = Item::new("an-item");
/// # tokio_test::block_on(async {
///     stac::io::put("s3://bucket/an-item.json", item).await.unwrap();
/// # })
/// }
/// ```
#[cfg(feature = "object-store")]
pub async fn put<T>(href: impl ToString, value: T) -> Result<Option<object_store::PutResult>>
where
    T: ToJson + ToNdjson + IntoGeoparquet,
{
    let options: [(&str, &str); 0] = [];
    put_opts(href, value, options).await
}

/// Puts a value, maybe to an object store with options.
///
/// # Examples
///
/// ```no_run
/// use stac::Item;
///
/// #[cfg(feature = "object-store-aws")]
/// {
/// let item = Item::new("an-item");
/// # tokio_test::block_on(async {
///     stac::io::put_opts("s3://bucket/an-item.json", item, [("aws_access_key_id", "...")]).await.unwrap();
/// # })
/// }
/// ```
#[cfg(feature = "object-store")]
pub async fn put_opts<T, I, K, V>(
    href: impl ToString,
    value: T,
    options: I,
) -> Result<Option<object_store::PutResult>>
where
    T: ToJson + ToNdjson + IntoGeoparquet,
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    let href = href.to_string();
    let format = Format::infer_from_href(&href).unwrap_or_default();
    format.put_opts(href, value, options).await
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use crate::{Catalog, Collection, Item, ItemCollection};

    macro_rules! read {
        ($function:ident, $filename:expr, $value:ty $(, $meta:meta)?) => {
            #[test]
            $(#[$meta])?
            fn $function() {
                use crate::SelfHref;

                let value: $value = crate::read($filename).unwrap();
                assert!(value.self_href().is_some());
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
    mod read_with_reqwest {
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

    #[test]
    #[cfg(not(feature = "reqwest"))]
    fn read_without_reqwest() {
        assert!(matches!(
            super::read::<Item>("http://stac-rs.test/item.json").unwrap_err(),
            crate::Error::FeatureNotEnabled("reqwest")
        ));
    }

    #[test]
    #[cfg(feature = "geoparquet")]
    fn read_geoparquet() {
        let _: ItemCollection = super::read("data/extended-item.parquet").unwrap();
    }

    #[test]
    #[cfg(not(feature = "geoparquet"))]
    fn read_geoparquet_without_geoparquet() {
        let _ = super::read::<ItemCollection>("data/extended-item.parquet").unwrap_err();
    }

    #[tokio::test]
    #[cfg(all(feature = "object-store", not(target_os = "windows")))]
    async fn get() {
        let path = format!(
            "file://{}",
            std::fs::canonicalize("examples/simple-item.json")
                .unwrap()
                .to_string_lossy()
        );
        let _: Item = super::get(path).await.unwrap();
    }

    #[test]
    fn write() {
        let tempdir = TempDir::new().unwrap();
        let item = Item::new("an-id");
        super::write(tempdir.path().join("item.json"), item).unwrap();
        let item: Item = super::read(tempdir.path().join("item.json")).unwrap();
        assert_eq!(item.id, "an-id");
    }

    #[tokio::test]
    #[cfg(feature = "object-store")]
    async fn put() {
        let tempdir = TempDir::new().unwrap();
        let path = format!(
            "file://{}",
            tempdir.path().join("item.json").to_string_lossy()
        );
        let item = Item::new("an-id");
        assert!(super::put(path, item).await.unwrap().is_some());
        let item: Item = crate::read(tempdir.path().join("item.json")).unwrap();
        assert_eq!(item.id, "an-id");
    }
}
