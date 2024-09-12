//! Structures for writing output data.

use crate::{config::Config, value::Value, Error, Result};
use object_store::{buffered::BufWriter, path::Path, ObjectStore, PutPayload, PutResult};
use std::{fmt::Display, io::IsTerminal, pin::Pin, str::FromStr, sync::Arc};
use tokio::{
    fs::File,
    io::{AsyncWrite, AsyncWriteExt},
};
use url::Url;

/// The output from a CLI run.
#[allow(missing_debug_implementations)]
pub(crate) struct Output {
    /// The output format.
    pub format: Format,

    writer: Writer,
}

/// The output format.
#[derive(Clone, Debug, PartialEq, Copy)]
pub(crate) enum Format {
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
    pub(crate) async fn new(
        outfile: Option<String>,
        format: Option<Format>,
        config: impl Into<Config>,
        create_parent_directories: bool,
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
            if let Some(url) = Url::parse(&outfile).ok().and_then(|url| {
                if url.scheme() == "file" {
                    None
                } else {
                    Some(url)
                }
            }) {
                let (object_store, path) =
                    object_store::parse_url_opts(&url, config.into().iter())?;
                let object_store = Arc::new(object_store);
                let stream = BufWriter::new(object_store.clone(), path.clone());
                Writer {
                    stream: Box::pin(stream),
                    object_store: Some((object_store, path)),
                }
            } else {
                if create_parent_directories {
                    if let Some(parent) = std::path::Path::new(&outfile).parent() {
                        std::fs::create_dir_all(parent)?;
                    }
                }
                Writer {
                    stream: Box::pin(File::create(outfile).await?),
                    object_store: None,
                }
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
    pub(crate) async fn stream(&mut self, value: Value) -> Result<()> {
        let bytes = value.into_bytes(Format::NdJson)?;
        self.writer.stream.write_all(&bytes).await?;
        self.writer.stream.flush().await?;
        Ok(())
    }

    /// Puts a value to the output.
    pub(crate) async fn put(&mut self, value: Value) -> Result<Option<PutResult>> {
        let bytes = value.into_bytes(self.format)?;
        if let Some((object_store, path)) = &self.writer.object_store {
            object_store
                .put(path, PutPayload::from_bytes(bytes))
                .await
                .map(Some)
                .map_err(Error::from)
        } else {
            let output = self.writer.stream.write_all(&bytes).await.map(|_| None)?;
            self.writer.stream.flush().await?;
            Ok(output)
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
                        if let Some(stop) = compression.find(']') {
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

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "geoparquet")]
    fn geoparquet_compression() {
        let format: super::Format = "geoparquet[snappy]".parse().unwrap();
        assert_eq!(
            format,
            super::Format::Geoparquet(parquet::basic::Compression::SNAPPY)
        );
    }
}
