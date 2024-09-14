use crate::{Error, ItemCollection, Object, Result};
#[cfg(feature = "geoparquet")]
use parquet::basic::Compression;
use std::{fmt::Display, io::Write, str::FromStr};

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
pub trait FormatIntoBytes {
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
    fn format_into_bytes(self, format: Format) -> Result<Vec<u8>>;
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

impl<O: Object> FormatIntoBytes for O {
    fn format_into_bytes(self, format: Format) -> Result<Vec<u8>> {
        match format {
            Format::Json(pretty) => {
                if pretty {
                    serde_json::to_vec_pretty(&self).map_err(Error::from)
                } else {
                    serde_json::to_vec(&self).map_err(Error::from)
                }
            }
            Format::NdJson => {
                let mut buf = Vec::new();
                self.ndjson_to_writer(&mut buf)?;
                Ok(buf)
            }
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(compression) => {
                let mut buf = Vec::new();
                self.geoparquet_into_writer(&mut buf, compression)?;
                Ok(buf)
            }
        }
    }
}

impl FormatIntoBytes for serde_json::Value {
    fn format_into_bytes(self, format: Format) -> Result<Vec<u8>> {
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
                let item_collection: ItemCollection = serde_json::from_value(self)?;
                let mut buf = Vec::new();
                item_collection.geoparquet_into_writer(&mut buf, compression)?;
                Ok(buf)
            }
        }
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
