use crate::{Error, Href, Result, Value};
use bytes::Bytes;
#[cfg(feature = "geoparquet")]
use parquet::basic::Compression;
use serde::de::DeserializeOwned;
use std::{
    fmt::Display,
    fs::File,
    io::{BufRead, BufReader, BufWriter, Read, Write},
    path::Path,
    str::FromStr,
};
use url::Url;

/// The format of STAC data.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Format {
    /// JSON data (the default).
    ///
    /// The boolean indicates whether it should be pretty-printed (true for pretty).
    Json(bool),

    /// Newline-delimited JSON.
    NdJson,

    /// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet)
    #[cfg(feature = "geoparquet")]
    Geoparquet(Option<Compression>),
}

/// A trait for things that can be converted into formatted bytes.
pub trait IntoFormattedBytes {
    /// Converts this value into formatted bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, io::{IntoFormattedBytes, Format}};
    ///
    /// let item = Item::new("an-id");
    /// let bytes = item.into_formatted_bytes(Format::Json(true)).unwrap();
    /// ```
    fn into_formatted_bytes(self, format: Format) -> Result<Vec<u8>>;
}

impl Format {
    /// Infer the format from a file extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::io::Format;
    ///
    /// assert_eq!(Format::Json(false), Format::infer_from_href("item.json").unwrap());
    /// ```
    pub fn infer_from_href(href: &str) -> Option<Format> {
        href.rsplit_once('.').and_then(|(_, ext)| ext.parse().ok())
    }

    /// Sets this format to the pretty version, if possible.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::io::Format;
    ///
    /// assert_eq!(Format::Json(false).pretty(), Format::Json(true));
    /// ```
    pub fn pretty(self) -> Format {
        if let Format::Json(_) = self {
            Format::Json(true)
        } else {
            self
        }
    }

    /// Reads a STAC value from a href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, io::Format};
    ///
    /// let item: Item = Format::Json(false).read("examples/simple-item.json").unwrap();
    /// ```
    #[cfg_attr(not(feature = "reqwest"), allow(unused_variables))]
    pub fn read<T>(&self, href: impl ToString) -> Result<T>
    where
        T: Href + DeserializeOwned,
    {
        let href = href.to_string();
        let mut value: T = if let Some(url) = href_to_url(&href) {
            #[cfg(feature = "reqwest")]
            {
                let response = reqwest::blocking::get(url.clone())?;
                self.from_bytes(response.bytes()?)?
            }
            #[cfg(not(feature = "reqwest"))]
            {
                return Err(Error::ReqwestNotEnabled);
            }
        } else {
            let mut file = File::open(&href)?;
            if matches!(self, Format::NdJson) {
                let mut array = Vec::new();
                for line in BufReader::new(file).lines() {
                    array.push(serde_json::to_value(line?)?);
                }
                serde_json::from_value(serde_json::Value::Array(array))?
            } else {
                let mut buf = Vec::new();
                let _ = file.read_to_end(&mut buf)?;
                self.from_bytes(buf.into())?
            }
        };
        value.set_href(href);
        Ok(value)
    }

    /// Reads a STAC value from [Bytes].
    ///
    /// For geoparquet, this will be less efficient than using
    /// [crate::io::geoparquet::from_reader] because we have to round-trip
    /// serialization to get the `T` output type.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, io::Format};
    /// use std::{io::Read, fs::File};
    ///
    /// let mut file = File::open("examples/simple-item.json").unwrap();
    /// let mut buf = Vec::new();
    /// file.read_to_end(&mut buf).unwrap();
    /// let item: Item = Format::Json(false).from_bytes(buf.into()).unwrap();
    /// ```
    pub fn from_bytes<T: DeserializeOwned>(&self, bytes: Bytes) -> Result<T> {
        match self {
            Format::Json(_) => serde_json::from_slice(&bytes).map_err(Error::from),
            Format::NdJson => bytes
                .split(|b| *b == b'\n')
                .filter_map(|line| {
                    if line.is_empty() {
                        None
                    } else {
                        Some(serde_json::from_slice(line))
                    }
                })
                .collect::<std::result::Result<Vec<serde_json::Value>, _>>()
                .map(serde_json::Value::Array)
                .and_then(serde_json::from_value)
                .map_err(Error::from),
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => {
                let item_collection = crate::io::geoparquet::from_reader(bytes)?;
                serde_json::to_value(item_collection)
                    .and_then(serde_json::from_value)
                    .map_err(Error::from)
            }
        }
    }

    /// Writes a [Value] to a [Write].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{io::Format, Item};
    ///
    /// let mut buf = Vec::new();
    /// Format::Json(true).to_writer(&mut buf, Item::new("an-id")).unwrap();
    /// ```
    pub fn to_writer<W: Write + Send>(&self, mut writer: W, value: impl Into<Value>) -> Result<()> {
        let value = value.into();
        match self {
            Format::Json(pretty) => {
                if *pretty {
                    serde_json::to_writer_pretty(writer, &value).map_err(Error::from)
                } else {
                    serde_json::to_writer(writer, &value).map_err(Error::from)
                }
            }
            Format::NdJson => {
                if let Value::ItemCollection(item_collection) = value {
                    for item in &item_collection.items {
                        serde_json::to_writer(&mut writer, item)?;
                        writeln!(&mut writer)?;
                    }
                    Ok(())
                } else {
                    serde_json::to_writer(writer, &value).map_err(Error::from)
                }
            }
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(compression) => {
                if let Some(compression) = compression {
                    crate::io::geoparquet::to_writer_with_compression(writer, value, *compression)
                } else {
                    crate::io::geoparquet::to_writer(writer, value)
                }
            }
        }
    }

    /// Writes a [IntoFormattedBytes] to a href.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::{Item, io::Format};
    ///
    /// let item = Item::new("an-id");
    /// Format::Json(true).write("an-id.json", item).unwrap();
    /// ```
    pub fn write(&self, href: impl AsRef<Path>, value: impl IntoFormattedBytes) -> Result<()> {
        let bytes = value.into_formatted_bytes(*self)?;
        let mut file = BufWriter::new(File::create(href)?);
        file.write_all(&bytes).map_err(Error::from)
    }
}

impl Default for Format {
    fn default() -> Self {
        Self::Json(false)
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Json(pretty) => {
                if *pretty {
                    f.write_str("json-pretty")
                } else {
                    f.write_str("json")
                }
            }
            Self::NdJson => f.write_str("ndjson"),
            #[cfg(feature = "geoparquet")]
            Self::Geoparquet(compression) => {
                if let Some(compression) = *compression {
                    write!(f, "geoparquet[{}]", compression)
                } else {
                    f.write_str("geoparquet")
                }
            }
        }
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Format> {
        match s.to_ascii_lowercase().as_str() {
            "json" | "geojson" => Ok(Self::Json(false)),
            "json-pretty" | "geojson-pretty" => Ok(Self::Json(true)),
            "ndjson" => Ok(Self::NdJson),
            _ => {
                if s.starts_with("parquet") || s.starts_with("geoparquet") {
                    #[cfg(feature = "geoparquet")]
                    if let Some((_, compression)) = s.split_once('[') {
                        if let Some(stop) = compression.find(']') {
                            Ok(Self::Geoparquet(Some(compression[..stop].parse()?)))
                        } else {
                            Err(Error::UnsupportedFormat(s.to_string()))
                        }
                    } else {
                        Ok(Self::Geoparquet(None))
                    }
                    #[cfg(not(feature = "geoparquet"))]
                    {
                        log::warn!("{} has a geoparquet extension, but the geoparquet feature is not enabled", s);
                        Err(Error::UnsupportedFormat(s.to_string()))
                    }
                } else {
                    Err(Error::UnsupportedFormat(s.to_string()))
                }
            }
        }
    }
}

impl<T> IntoFormattedBytes for T
where
    T: Into<Value>,
{
    fn into_formatted_bytes(self, format: Format) -> Result<Vec<u8>> {
        let value = self.into();
        let mut buf = Vec::new();
        format.to_writer(&mut buf, value)?;
        Ok(buf)
    }
}

impl IntoFormattedBytes for serde_json::Value {
    fn into_formatted_bytes(self, format: Format) -> Result<Vec<u8>> {
        match format {
            Format::Json(pretty) => {
                if pretty {
                    serde_json::to_vec_pretty(&self).map_err(Error::from)
                } else {
                    serde_json::to_vec(&self).map_err(Error::from)
                }
            }
            Format::NdJson => {
                if let serde_json::Value::Array(array) = self {
                    let mut buf = Vec::new();
                    for value in array {
                        serde_json::to_writer(&mut buf, &value)?;
                        writeln!(&mut buf)?;
                    }
                    Ok(buf)
                } else {
                    serde_json::to_vec(&self).map_err(Error::from)
                }
            }
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(compression) => {
                let item_collection: crate::ItemCollection = serde_json::from_value(self)?;
                let mut buf = Vec::new();
                if let Some(compression) = compression {
                    crate::io::geoparquet::to_writer_with_compression(
                        &mut buf,
                        item_collection,
                        compression,
                    )?;
                } else {
                    crate::io::geoparquet::to_writer(&mut buf, item_collection)?;
                }
                Ok(buf)
            }
        }
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

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "geoparquet")]
    fn parse_geoparquet() {
        assert_eq!(
            "parquet".parse::<super::Format>().unwrap(),
            super::Format::Geoparquet(None)
        );
    }

    #[test]
    #[cfg(feature = "geoparquet")]
    fn parse_geoparquet_compression() {
        let format: super::Format = "geoparquet[snappy]".parse().unwrap();
        assert_eq!(
            format,
            super::Format::Geoparquet(Some(parquet::basic::Compression::SNAPPY))
        );
    }
}
