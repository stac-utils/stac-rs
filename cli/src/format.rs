use crate::{Error, Result, Value};
use stac::{Item, ItemCollection};
use std::{
    io::{BufRead, BufReader, Read, Write},
    str::FromStr,
};

/// A file format that we can read and/or write.
#[derive(Clone, Debug, PartialEq)]
pub enum Format {
    /// Pretty-printed JSON
    PrettyJson,

    /// Compact-printed JSON
    CompactJson,

    /// Streaming data.
    ///
    /// For JSON, this is newline-delimited JSON.
    Streaming,

    /// stac-geoparquet
    #[cfg(feature = "geoparquet")]
    Geoparquet(Option<parquet::basic::Compression>),
}

impl FromStr for Format {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self> {
        match s {
            "json" | "pretty-json" | "pretty" => Ok(Self::PrettyJson),
            "compact-json" | "compact" => Ok(Self::CompactJson),
            "nd-json" | "ndjson" | "streaming" | "stream" => Ok(Self::Streaming),
            // TODO we could embed compression in these strings
            "geoparquet" | "parquet" | "stac-geoparquet" => {
                #[cfg(feature = "geoparquet")]
                {
                    Ok(Self::Geoparquet(None))
                }
                #[cfg(not(feature = "geoparquet"))]
                {
                    tracing::error!(
                        "geoparquet was requested, but the `geoparquet` feature is not enabled"
                    );
                    Err(Error::UnsupportedFormat(s.to_string()))
                }
            }
            _ => Err(Error::UnsupportedFormat(s.to_string())),
        }
    }
}

impl Format {
    /// Infers the format from the file extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_cli::Format;
    ///
    /// assert_eq!(Format::infer("item.json").unwrap(), Format::CompactJson);
    /// ```
    pub fn infer(href: &str) -> Option<Format> {
        href.rsplit_once('.').and_then(|(_, ext)| match ext {
            "json" | "geojson" => Some(Format::CompactJson),
            "parquet" | "geoparquet" => {
                #[cfg(feature = "geoparquet")]
                {
                    Some(Format::Geoparquet(None))
                }
                #[cfg(not(feature = "geoparquet"))]
                {
                    tracing::warn!("'{}' has a geoparquet file extension, but the geoparquet feature is not enabled — falling back to compact json", href);
                    Some(Format::CompactJson)
                }
            }
            _ => None
        })
    }

    /// Writes a [Value] to a [Write].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_cli::Format;
    ///
    /// let item: stac::Value = stac::read("examples/simple-item.json").unwrap();
    /// Format::PrettyJson.to_writer(std::io::stdout(), item).unwrap();
    /// ```
    pub fn to_writer(&self, mut writer: impl Write + Send, value: impl Into<Value>) -> Result<()> {
        match self {
            Format::PrettyJson => {
                serde_json::to_writer_pretty(writer, &value.into()).map_err(Error::from)
            }
            Format::CompactJson => {
                serde_json::to_writer(writer, &value.into()).map_err(Error::from)
            }
            Format::Streaming => {
                let value = value.into();
                match value {
                    Value::Stac(value) => {
                        if let stac::Value::ItemCollection(item_collection) = value {
                            for item in item_collection {
                                serde_json::to_writer(&mut writer, &item)?;
                                writeln!(&mut writer)?;
                            }
                            Ok(())
                        } else {
                            serde_json::to_writer(&mut writer, &value)?;
                            writeln!(writer).map_err(Error::from)
                        }
                    }
                    Value::Json(value) => {
                        if let serde_json::Value::Array(array) = value {
                            for value in array {
                                serde_json::to_writer(&mut writer, &value)?;
                                writeln!(&mut writer)?;
                            }
                            Ok(())
                        } else {
                            serde_json::to_writer(&mut writer, &value)?;
                            writeln!(writer).map_err(Error::from)
                        }
                    }
                    Value::String(string) => writeln!(writer, "{}", string).map_err(Error::from),
                }
            }
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(compression) => {
                let value = stac::Value::try_from(value.into())?;
                let mut options = geoarrow::io::parquet::GeoParquetWriterOptions::default();
                if let Some(compression) = compression {
                    let writer_properties = parquet::file::properties::WriterProperties::builder()
                        .set_compression(*compression)
                        .build();
                    options.writer_properties = Some(writer_properties);
                }
                stac::geoparquet::to_writer_with_options(writer, value, &options)
                    .map_err(Error::from)
            }
        }
    }

    /// Reads a [stac::Value] from a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_cli::Format;
    ///
    /// let item = Format::CompactJson.from_file("examples/simple-item.json").unwrap();
    /// ```
    pub fn from_file(&self, file: &str) -> Result<stac::Value> {
        match self {
            Format::CompactJson | Format::PrettyJson | Format::Streaming => {
                stac::io::json::read(file).map_err(Error::from)
            }
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => stac::io::geoparquet::read(file)
                .map(stac::Value::from)
                .map_err(Error::from),
        }
    }

    /// Reads a [stac::Value] from a [Read].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_cli::Format;
    ///
    /// let item = Format::CompactJson.from_file("examples/simple-item.json").unwrap();
    /// ```
    #[cfg_attr(not(feature = "geoparquet"), allow(unused_mut))]
    pub fn from_reader<R: Read>(&self, mut reader: R) -> Result<stac::Value> {
        match self {
            Format::CompactJson | Format::PrettyJson => {
                serde_json::from_reader(reader).map_err(Error::from)
            }
            Format::Streaming => {
                let mut items = Vec::new();
                for line in BufReader::new(reader).lines() {
                    let item: Item = serde_json::from_str(&line?)?;
                    items.push(item);
                }
                Ok(ItemCollection::from(items).into())
            }
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(_) => {
                use bytes::Bytes;

                let mut buf = Vec::new();
                let _ = reader.read_to_end(&mut buf)?;
                let bytes = Bytes::from(buf);
                stac::geoparquet::from_reader(bytes)
                    .map(stac::Value::from)
                    .map_err(Error::from)
            }
        }
    }
}
