//! Read data from and write data to [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet/blob/main/spec/stac-geoparquet-spec.md).

use crate::Result;
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

#[cfg(feature = "geoparquet")]
mod feature;
#[cfg(not(feature = "geoparquet"))]
mod no_feature;

use bytes::Bytes;
#[cfg(not(feature = "geoparquet"))]
pub use no_feature::Compression;
#[cfg(feature = "geoparquet")]
pub use {
    feature::{from_reader, into_writer, into_writer_with_compression, into_writer_with_options},
    parquet::basic::Compression,
};

/// Create a STAC object from geoparquet data.
pub trait FromGeoparquet: Sized {
    /// Reads geoparquet data from a file.
    ///
    /// If the `geoparquet` feature is not enabled, or if `Self` is anything
    /// other than an item collection, this function returns an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{FromGeoparquet, ItemCollection};
    ///
    /// #[cfg(feature = "geoparquet")]
    /// {
    ///     let item_collection = ItemCollection::from_geoparquet_path("data/extended-item.parquet").unwrap();
    /// }
    /// ```
    fn from_geoparquet_path(path: impl AsRef<Path>) -> Result<Self> {
        let mut buf = Vec::new();
        let _ = File::open(path)?.read_to_end(&mut buf)?;
        Self::from_geoparquet_bytes(buf)
    }

    /// Creates a STAC object from geoparquet bytes.
    #[allow(unused_variables)]
    fn from_geoparquet_bytes(bytes: impl Into<Bytes>) -> Result<Self>;
}

/// Write a STAC object to geoparquet.
pub trait IntoGeoparquet: Sized {
    /// Writes a value to a path as stac-geoparquet.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::{IntoGeoparquet, ItemCollection, Item};
    ///
    /// let item_collection: ItemCollection = vec![Item::new("a"), Item::new("b")].into();
    /// item_collection.into_geoparquet_path("items.geoparquet", None).unwrap();
    /// ```
    fn into_geoparquet_path(
        self,
        path: impl AsRef<Path>,
        compression: Option<Compression>,
    ) -> Result<()> {
        let file = File::create(path)?;
        self.into_geoparquet_writer(file, compression)
    }

    /// Writes a value to a writer as stac-geoparquet.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::{IntoGeoparquet, ItemCollection, Item};
    ///
    /// let item_collection: ItemCollection = vec![Item::new("a"), Item::new("b")].into();
    /// let mut buf = Vec::new();
    /// item_collection.into_geoparquet_writer(&mut buf, None).unwrap();
    /// ```
    fn into_geoparquet_writer(
        self,
        writer: impl Write + Send,
        compression: Option<Compression>,
    ) -> Result<()>;

    /// Writes a value to a writer as stac-geoparquet to some bytes.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::{IntoGeoparquet, ItemCollection, Item};
    ///
    /// let item_collection: ItemCollection = vec![Item::new("a"), Item::new("b")].into();
    /// let bytes = item_collection.into_geoparquet_vec(None).unwrap();
    /// ```
    fn into_geoparquet_vec(self, compression: Option<Compression>) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.into_geoparquet_writer(&mut buf, compression)?;
        Ok(buf)
    }
}

macro_rules! impl_from_geoparquet {
    ($object:ty) => {
        impl FromGeoparquet for $object {
            fn from_geoparquet_bytes(
                _: impl Into<Bytes>,
            ) -> std::result::Result<Self, crate::Error> {
                #[cfg(feature = "geoparquet")]
                {
                    Err(crate::Error::UnsupportedGeoparquetType)
                }
                #[cfg(not(feature = "geoparquet"))]
                {
                    Err(crate::Error::FeatureNotEnabled("geoparquet"))
                }
            }
        }
    };
}
macro_rules! impl_into_geoparquet {
    ($object:ty) => {
        impl IntoGeoparquet for $object {
            fn into_geoparquet_writer(
                self,
                _: impl Write + Send,
                _: Option<Compression>,
            ) -> std::result::Result<(), crate::Error> {
                #[cfg(feature = "geoparquet")]
                {
                    Err(crate::Error::UnsupportedGeoparquetType)
                }
                #[cfg(not(feature = "geoparquet"))]
                {
                    Err(crate::Error::FeatureNotEnabled("geoparquet"))
                }
            }
        }
    };
}

impl_from_geoparquet!(crate::Item);
impl_from_geoparquet!(crate::Catalog);
impl_from_geoparquet!(crate::Collection);
impl_into_geoparquet!(crate::Catalog);
impl_into_geoparquet!(crate::Collection);

pub(crate) use impl_from_geoparquet;
pub(crate) use impl_into_geoparquet;
