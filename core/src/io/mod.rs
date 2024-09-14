//! Input and output.
//!
//! # Reading
//!
//! Basic reads are provided by [read]:
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
//! #[cfg(feature = "geoparquet")]
//! {
//!     let item_collection: stac::ItemCollection = stac::read("data/extended-item.parquet").unwrap();
//! }
//! ```
//!
//! To avoid inferring the format from the href's extension, use [Format::read].
//!
//! ## Geoparquet performance
//!
//! [read] and [Format::read] both return `T: Href + DeserializeOwned`. When reading `geoparquet`,
//! this requires round-tripping the resultant item collection through serialization to get a `T`.
//! If you know you're reading geoparquet, prefer [geoparquet::from_reader].
//!
//! # Writing
//!
//! Use [Format::to_writer]:
//!
//! ```
//! use stac::{Item, io::Format};
//!
//! let item = Item::new("an-id");
//! let mut buf = Vec::new();
//! Format::Json(true).to_writer(&mut buf, item).unwrap();
//! ```
//!
//! # Object store
//!
//! If the `object-store` feature is enabled, use [get_format_opts] and
//! [put_format_opts] to get and put values from an object store.

mod format;
#[cfg(feature = "geoarrow")]
pub mod geoarrow;
#[cfg(feature = "geoparquet")]
pub mod geoparquet;

use crate::{Object, Result};
pub use format::{Format, FormatIntoBytes};
use std::{fs::File, io::Write};

/// Reads any STAC value from an href.
///
/// # Examples
///
/// ```
/// let item: stac::Item = stac::read("examples/simple-item.json").unwrap();
/// ```
pub fn read<T: Object>(href: impl ToString) -> Result<T> {
    let href = href.to_string();
    let format = Format::infer_from_href(&href).unwrap_or_default();
    T::format_read(format, href)
}

/// Writes any STAC value to an href.
///
/// # Examples
///
/// ```no_run
/// let item = stac::Item::new("an-id");
/// stac::write("an-id.json", item).unwrwap();
/// ```
pub fn write<T: Object>(href: impl ToString, value: T) -> Result<()> {
    let href = href.to_string();
    let format = Format::infer_from_href(&href).unwrap_or_default();
    value.format_write(format, href)
}

/// Gets a value, maybe from an object store.
///
/// If `href` is a url, [object_store::parse_url_opts] will be used to build the
/// object store to get the value. Otherwise, this is just forwarded on to
/// [Format::read].
///
/// If `format` is `None`, the format will be inferred from the href.
///
/// # Examples
///
/// ```
/// use stac::io::Format;
///
/// #[cfg(feature = "object-store")]
/// {
/// # tokio_test::block_on(async {
///     let item: stac::Item = stac::io::get_format_opts("examples/simple-item.json", Format::Json(false), [("foo", "bar")]).await.unwrap();
/// # })
/// }
/// ```
#[cfg(feature = "object-store")]
pub async fn get_format_opts<T, I, K, V>(
    href: impl ToString,
    format: impl Into<Option<Format>>,
    options: I,
) -> Result<T>
where
    T: Object,
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    let href = href.to_string();
    let format = format
        .into()
        .or_else(|| Format::infer_from_href(&href))
        .unwrap_or_default();
    if let Ok(url) = url::Url::parse(&href) {
        use object_store::ObjectStore;

        let (object_store, path) = object_store::parse_url_opts(&url, options)?;
        let get_result = object_store.get(&path).await?;
        let mut value: T = T::format_from_bytes(format, get_result.bytes().await?)?;
        *value.href_mut() = Some(href);
        Ok(value)
    } else {
        T::format_read(format, href)
    }
}

/// Puts a value, maybe to an object store.
///
/// If `href` is a url, [object_store::parse_url_opts] will be used to build the
/// object store to put the value to. Otherwise, this is just forwarded on to
/// [Format::write].
///
/// If `format` is `None`, the format will be inferred from the href.
///
/// # Examples
///
/// ```
/// use stac::{Item, io::Format};
///
/// #[cfg(feature = "object-store")]
/// {
/// let item = Item::new("an-item");
/// # tokio_test::block_on(async {
///     stac::io::put_format_opts("an-item.json", item, Format::Json(false), [("foo", "bar")]).await.unwrap();
/// # })
/// }
/// ```
#[cfg(feature = "object-store")]
pub async fn put_format_opts<I, K, V>(
    href: impl ToString,
    value: impl FormatIntoBytes,
    format: impl Into<Option<Format>>,
    options: I,
) -> Result<Option<object_store::PutResult>>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    let href = href.to_string();
    let format = format
        .into()
        .or_else(|| Format::infer_from_href(&href))
        .unwrap_or_default();
    let bytes = value.format_into_bytes(format)?;
    if let Ok(url) = url::Url::parse(&href) {
        use object_store::ObjectStore;

        let (object_store, path) = object_store::parse_url_opts(&url, options)?;
        object_store
            .put(&path, bytes.into())
            .await
            .map(Some)
            .map_err(crate::Error::from)
    } else {
        let mut file = File::create(href)?;
        file.write_all(&bytes)?;
        Ok(None)
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
                use crate::Object;

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
}
