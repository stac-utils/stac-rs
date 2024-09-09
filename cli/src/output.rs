//! Structures for writing output data.

use crate::{Config, Error, Result, Value};
use object_store::{buffered::BufWriter, path::Path, ObjectStore, PutPayload, PutResult};
use std::{fmt::Display, io::IsTerminal, pin::Pin, str::FromStr, sync::Arc};
use tokio::io::{AsyncWrite, AsyncWriteExt};

/// The output from a CLI run.
#[allow(missing_debug_implementations)]
pub struct Output {
    /// The output format.
    pub format: Format,

    writer: Writer,
}

/// The output format.
#[derive(Clone, Debug, PartialEq)]
pub enum Format {
    /// Pretty-printed JSON, good for terminals.
    PrettyJson,
    /// Compact JSON, good for files.
    CompactJson,
    /// Newline-delimited JSON
    NdJson,
    /// stac-geoparquet
    #[cfg(feature = "geoparquet")]
    Geoparquet(parquet::basic::Compression),
}

struct Writer {
    stream: Pin<Box<dyn AsyncWrite + Send>>,
    object_store: Option<(Arc<Box<dyn ObjectStore>>, Path)>,
}

impl Output {
    /// Creates a new output from an optional outfile and an optional format.
    pub fn new(
        outfile: Option<String>,
        format: Option<Format>,
        config: impl Into<Config>,
    ) -> Result<Output> {
        let format = format
            .or_else(|| outfile.as_deref().and_then(Format::infer_from_href))
            .unwrap_or_else(|| {
                if outfile.is_some() && !std::io::stdout().is_terminal() {
                    Format::CompactJson
                } else {
                    Format::PrettyJson
                }
            });
        let writer = if let Some(outfile) = outfile {
            let (object_store, path) =
                crate::object_store::parse_href_opts(&outfile, config.into().iter())?;
            let object_store = Arc::new(object_store);
            let stream = BufWriter::new(object_store.clone(), path.clone());
            Writer {
                stream: Box::pin(stream),
                object_store: Some((object_store, path)),
            }
        } else {
            Writer {
                stream: Box::pin(tokio::io::stdout()),
                object_store: None,
            }
        };
        Ok(Output { format, writer })
    }

    /// Streams a value to the output
    pub async fn stream(&mut self, value: Value) -> Result<()> {
        match value {
            Value::Json(ref json) => {
                if let serde_json::Value::Array(array) = json {
                    for value in array {
                        let value = serde_json::to_vec(value)?;
                        let _ = self.writer.stream.write_all(&value).await?;
                        self.writer.stream.write_u8(b'\n').await?;
                    }
                } else {
                    let value = serde_json::to_vec(&value)?;
                    let _ = self.writer.stream.write_all(&value).await?;
                    self.writer.stream.write_u8(b'\n').await?;
                }
            }
            Value::Stac(stac) => {
                if let stac::Value::ItemCollection(item_collection) = stac {
                    for value in item_collection.items {
                        let value = serde_json::to_vec(&value)?;
                        let _ = self.writer.stream.write_all(&value).await?;
                        self.writer.stream.write_u8(b'\n').await?;
                    }
                } else {
                    let value = serde_json::to_vec(&stac)?;
                    let _ = self.writer.stream.write_all(&value).await?;
                    self.writer.stream.write_u8(b'\n').await?;
                }
            }
        }
        self.writer.stream.flush().await?;
        Ok(())
    }

    /// Puts a value to the output.
    pub async fn put(&mut self, value: Value) -> Result<Option<PutResult>> {
        if let Some((object_store, path)) = &self.writer.object_store {
            let bytes = match self.format {
                Format::PrettyJson => serde_json::to_vec_pretty(&value)?,
                Format::CompactJson => serde_json::to_vec(&value)?,
                Format::NdJson => {
                    let mut bytes = Vec::new();
                    match value {
                        Value::Json(ref json) => {
                            if let serde_json::Value::Array(array) = json {
                                for value in array {
                                    serde_json::to_writer(&mut bytes, value)?;
                                    bytes.push(b'\n');
                                }
                            } else {
                                serde_json::to_writer(&mut bytes, json)?;
                                bytes.push(b'\n');
                            }
                        }
                        Value::Stac(stac) => {
                            if let stac::Value::ItemCollection(item_collection) = stac {
                                for value in item_collection.items {
                                    serde_json::to_writer(&mut bytes, &value)?;
                                    bytes.push(b'\n');
                                }
                            } else {
                                serde_json::to_writer(&mut bytes, &stac)?;
                                bytes.push(b'\n');
                            }
                        }
                    }
                    todo!()
                }
                #[cfg(feature = "geoparquet")]
                Format::Geoparquet(compression) => {
                    let value = stac::Value::try_from(value)?;
                    let mut options = geoarrow::io::parquet::GeoParquetWriterOptions::default();
                    let writer_properties = parquet::file::properties::WriterProperties::builder()
                        .set_compression(compression)
                        .build();
                    options.writer_properties = Some(writer_properties);
                    let mut bytes = Vec::new();
                    stac::geoparquet::to_writer_with_options(&mut bytes, value, &options)?;
                    bytes
                }
            };
            object_store
                .put(path, PutPayload::from_bytes(bytes.into()))
                .await
                .map(Some)
                .map_err(Error::from)
        } else {
            match self.format {
                Format::PrettyJson => {
                    let value = serde_json::to_vec_pretty(&value)?;
                    self.writer.stream.write_all(&value).await?;
                }
                Format::CompactJson => {
                    let value = serde_json::to_vec(&value)?;
                    self.writer.stream.write_all(&value).await?;
                }
                Format::NdJson => {
                    self.stream(value).await?;
                }
                #[cfg(feature = "geoparquet")]
                Format::Geoparquet(compression) => {
                    // We cheat here b/c we know it's stdout ... could bite us later
                    let value = stac::Value::try_from(value)?;
                    let mut options = geoarrow::io::parquet::GeoParquetWriterOptions::default();
                    let writer_properties = parquet::file::properties::WriterProperties::builder()
                        .set_compression(compression)
                        .build();
                    options.writer_properties = Some(writer_properties);
                    stac::geoparquet::to_writer_with_options(std::io::stdout(), value, &options)?;
                }
            }
            self.writer.stream.flush().await?;
            Ok(None)
        }
    }
}

impl FromStr for Format {
    type Err = Error;
    fn from_str(s: &str) -> Result<Format> {
        let s = s.to_ascii_lowercase();
        match s.as_str() {
            "json" | "geojson" => Ok(Self::CompactJson),
            "ndjson" => Ok(Self::NdJson),
            _ => {
                #[cfg(feature = "geoparquet")]
                if s.starts_with("parquet") || s.starts_with("geoparquet") {
                    if let Some((_, compression)) = s.split_once('[') {
                        if let Some(stop) = s.find(']') {
                            Ok(Self::Geoparquet(compression[..stop].parse()?))
                        } else {
                            Err(Error::UnsupportedFormat(s.to_string()))
                        }
                    } else {
                        Ok(Self::Geoparquet(parquet::basic::Compression::UNCOMPRESSED))
                    }
                } else {
                    Err(Error::UnsupportedFormat(s.to_string()))
                }
                #[cfg(not(feature = "geoparquet"))]
                Err(Error::UnsupportedFormat(s.to_string()))
            }
        }
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Format::PrettyJson => f.write_str("pretty-json"),
            Format::CompactJson => f.write_str("compact-json"),
            Format::NdJson => f.write_str("nd-json"),
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(compression) => {
                if *compression == parquet::basic::Compression::UNCOMPRESSED {
                    f.write_str("geoparquet")
                } else {
                    write!(f, "geoparquet[{}]", compression)
                }
            }
        }
    }
}

impl Format {
    fn infer_from_href(href: &str) -> Option<Format> {
        href.rsplit_once('.').and_then(|(_, ext)| ext.parse().ok())
    }
}
