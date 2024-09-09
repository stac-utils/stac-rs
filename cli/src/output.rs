//! Structures for writing output data.

use crate::{Error, Result, Value};
use std::{
    fmt::Display,
    fs::File,
    io::{IsTerminal, Write},
    path::Path,
    str::FromStr,
};

/// The output from a CLI run.
#[allow(missing_debug_implementations)]
pub struct Output {
    /// The output format.
    pub format: Format,

    writer: Box<dyn Write + Send + 'static>,
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

impl Output {
    /// Creates a new output from an optional outfile and an optional format.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_cli::Output;
    ///
    /// let input = Output::new(None, None, false); // defaults to stdout, json, don't create directories
    /// ```
    pub fn new(
        outfile: Option<String>,
        format: Option<Format>,
        create_directories: bool,
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
        let writer: Box<dyn Write + Send> = if let Some(outfile) = outfile {
            let path = Path::new(&outfile);
            if create_directories && stac::href_to_url(&outfile).is_none() {
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            Box::new(File::create(path)?)
        } else {
            Box::new(std::io::stdout())
        };
        Ok(Output { format, writer })
    }

    /// Streams a value to the output
    pub fn stream(&mut self, value: Value) -> Result<()> {
        match value {
            Value::Json(json) => {
                if let serde_json::Value::Array(array) = json {
                    for value in array {
                        serde_json::to_writer(&mut self.writer, &value)?;
                        writeln!(&mut self.writer)?;
                    }
                } else {
                    serde_json::to_writer(&mut self.writer, &json)?;
                    writeln!(&mut self.writer)?;
                }
                Ok(())
            }
            Value::Stac(stac) => {
                if let stac::Value::ItemCollection(item_collection) = stac {
                    stac::ndjson::to_writer(&mut self.writer, item_collection.items.into_iter())?;
                } else {
                    serde_json::to_writer(&mut self.writer, &stac)?;
                    writeln!(&mut self.writer)?;
                }
                Ok(())
            }
        }
    }

    /// Puts a value to the output.
    pub fn put(&mut self, value: Value) -> Result<()> {
        match self.format {
            Format::PrettyJson => {
                serde_json::to_writer_pretty(&mut self.writer, &value).map_err(Error::from)
            }
            Format::CompactJson => {
                serde_json::to_writer(&mut self.writer, &value).map_err(Error::from)
            }
            Format::NdJson => self.stream(value),
            #[cfg(feature = "geoparquet")]
            Format::Geoparquet(compression) => {
                let value = stac::Value::try_from(value)?;
                let mut options = geoarrow::io::parquet::GeoParquetWriterOptions::default();
                let writer_properties = parquet::file::properties::WriterProperties::builder()
                    .set_compression(compression)
                    .build();
                options.writer_properties = Some(writer_properties);
                stac::geoparquet::to_writer_with_options(&mut self.writer, value, &options)
                    .map_err(Error::from)
            }
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
