use crate::{io::Format, Error, Result};
use bytes::Bytes;
#[cfg(feature = "geoparquet")]
use parquet::basic::Compression;
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
};
use url::Url;

/// A trait shared by all STAC objects, including [ItemCollection] and [Value].
///
/// # Examples
///
/// ```
/// use stac::{Item, Object};
///
/// let item = Item::new("an-id");
/// assert!(item.href().is_none());
/// let item: Item = stac::read("examples/simple-item.json").unwrap();
/// assert!(item.href().is_some());
/// ```
#[allow(missing_docs)]
pub trait Object: Sized + Serialize + DeserializeOwned {
    const TYPE: &str;

    /// Gets this object's href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Href, Item};
    ///
    /// let item: Item = stac::read("examples/simple-item.json").unwrap();
    /// assert_eq!(item.href(), Some("examples/simple-item.json"));
    /// ```
    fn href(&self) -> Option<&str>;

    /// Gets a mutable reference to this object's href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Object, Item};
    /// let item = Item::new("an-id");
    /// *item.href_mut() = Some("a/href.json".to_string());
    /// ```
    fn href_mut(&mut self) -> &mut Option<String>;

    /// Reads this object from the provided href in the given format.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Object, io::Format};
    ///
    /// let item = Item::format_read(Format::Json(false), "examples/simple-item.json").unwrap();
    /// ```
    #[cfg_attr(not(feature = "reqwest"), allow(unused_variables))]
    fn format_read(format: Format, href: impl ToString) -> Result<Self> {
        let href = href.to_string();
        let mut value = if let Some(url) = href_to_url(&href) {
            #[cfg(feature = "reqwest")]
            {
                let response = reqwest::blocking::get(url.clone())?;
                Self::format_from_bytes(format, response.bytes()?)?
            }
            #[cfg(not(feature = "reqwest"))]
            {
                return Err(Error::ReqwestNotEnabled);
            }
        } else {
            let file = File::open(&href)?;
            Self::format_from_file(format, file)?
        };
        *value.href_mut() = Some(href);
        Ok(value)
    }

    fn format_from_bytes(format: Format, bytes: impl Into<Bytes>) -> Result<Self> {
        match format {
            Format::Json(_) => serde_json::from_slice(&bytes.into()).map_err(Error::from),
            Format::NdJson => Self::ndjson_from_bytes(bytes),
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => Self::geoparquet_from_bytes(bytes),
        }
    }

    fn ndjson_from_bytes(bytes: impl Into<Bytes>) -> Result<Self> {
        serde_json::from_slice(&bytes.into()).map_err(Error::from)
    }

    #[cfg(feature = "geoparquet")]
    fn geoparquet_from_bytes(_: impl Into<Bytes>) -> Result<Self> {
        Err(Error::IncorrectType {
            actual: Self::TYPE.to_string(),
            expected: "ItemCollection".to_string(),
        })
    }

    fn format_from_file(format: Format, mut file: File) -> Result<Self> {
        match format {
            Format::Json(_) => {
                let mut buf = Vec::new();
                let _ = file.read_to_end(&mut buf)?;
                serde_json::from_slice(&buf).map_err(Error::from)
            }
            Format::NdJson => Self::ndjson_from_file(file),
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => Self::geoparquet_from_file(file),
        }
    }

    fn ndjson_from_file(mut file: File) -> Result<Self> {
        let mut buf = Vec::new();
        let _ = file.read_to_end(&mut buf)?;
        serde_json::from_slice(&buf).map_err(Error::from)
    }

    #[cfg(feature = "geoparquet")]
    fn geoparquet_from_file(_: File) -> Result<Self> {
        Err(Error::IncorrectType {
            actual: Self::TYPE.to_string(),
            expected: "ItemCollection".to_string(),
        })
    }

    fn format_write(self, format: Format, href: impl ToString) -> Result<()> {
        let href = href.to_string();
        let file = File::create(href)?;
        match format {
            Format::Json(pretty) => {
                if pretty {
                    serde_json::to_writer_pretty(file, &self).map_err(Error::from)
                } else {
                    serde_json::to_writer(file, &self).map_err(Error::from)
                }
            }
            Format::NdJson => self.ndjson_to_writer(file),
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(compression) => self.geoparquet_into_writer(file, compression),
        }
    }

    fn ndjson_to_writer(&self, mut writer: impl Write) -> Result<()> {
        serde_json::to_writer(&mut writer, &self)?;
        writeln!(writer)?;
        Ok(())
    }

    #[cfg(feature = "geoparquet")]
    fn geoparquet_into_writer(self, _: impl Write + Send, _: Option<Compression>) -> Result<()> {
        Err(Error::IncorrectType {
            actual: Self::TYPE.to_string(),
            expected: "ItemCollection".to_string(),
        })
    }
}

fn href_to_url(href: &str) -> Option<Url> {
    if let Ok(url) = Url::parse(href) {
        if url.scheme().starts_with("http") {
            Some(url)
        } else {
            None
        }
    } else {
        None
    }
}
