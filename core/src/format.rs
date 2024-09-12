use crate::{Error, Result, Value};
#[cfg(feature = "geoparquet")]
use parquet::basic::Compression;
use std::{fmt::Display, str::FromStr};

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

impl Format {
    /// Infer a format from a file extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Format;
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
    /// use stac::Format;
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

    /// Converts a [Value] to a vector of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Format, Item};
    ///
    /// let bytes = Format::Json(true).value_to_vec(Item::new("an-id")).unwrap();
    /// ```
    pub fn value_to_vec(&self, value: impl Into<Value>) -> Result<Vec<u8>> {
        let value = value.into();
        match self {
            Format::Json(pretty) => {
                if *pretty {
                    serde_json::to_vec_pretty(&value).map_err(Error::from)
                } else {
                    serde_json::to_vec(&value).map_err(Error::from)
                }
            }
            Format::NdJson => {
                if let Value::ItemCollection(item_collection) = value {
                    let mut buf = Vec::new();
                    for item in &item_collection.items {
                        serde_json::to_writer(&mut buf, item)?;
                        buf.push(b'\n');
                    }
                    Ok(buf)
                } else {
                    serde_json::to_vec(&value).map_err(Error::from)
                }
            }
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(compression) => {
                let mut options = geoarrow::io::parquet::GeoParquetWriterOptions::default();
                if let Some(compression) = compression {
                    let writer_properties = parquet::file::properties::WriterProperties::builder()
                        .set_compression(*compression)
                        .build();
                    options.writer_properties = Some(writer_properties);
                }
                let mut bytes = Vec::new();
                crate::geoparquet::to_writer_with_options(&mut bytes, value, &options)?;
                Ok(bytes)
            }
        }
    }

    /// Converts a [serde_json::Value] to a vector of bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Format;
    ///
    /// let data = serde_json::json!({
    ///     "foo": "bar"
    /// });
    /// let bytes = Format::Json(true).json_to_vec(data).unwrap();
    /// ```
    pub fn json_to_vec(&self, value: serde_json::Value) -> Result<Vec<u8>> {
        match self {
            Format::Json(pretty) => {
                if *pretty {
                    serde_json::to_vec_pretty(&value).map_err(Error::from)
                } else {
                    serde_json::to_vec(&value).map_err(Error::from)
                }
            }
            Format::NdJson => {
                if let serde_json::Value::Array(array) = value {
                    let mut buf = Vec::new();
                    for value in &array {
                        serde_json::to_writer(&mut buf, value)?;
                        buf.push(b'\n');
                    }
                    Ok(buf)
                } else {
                    serde_json::to_vec(&value).map_err(Error::from)
                }
            }
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => {
                let item_collection = crate::ItemCollection::try_from(value)?;
                self.value_to_vec(item_collection)
            }
        }
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
