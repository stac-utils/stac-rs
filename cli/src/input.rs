use crate::{Config, Error, Result};
use object_store::{path::Path, ObjectStore};
use stac::{Format, Item, ItemCollection, Value};
use std::io::BufReader;

/// The input to a CLI run.
#[derive(Debug)]
pub struct Input {
    format: Format,
    reader: Reader,
    config: Config,
}

#[derive(Debug)]
enum Reader {
    ObjectStore {
        object_store: Box<dyn ObjectStore>,
        path: Path,
    },
    Stdin,
}

impl Input {
    /// Creates a new input from an optional infile and an optional format.
    pub fn new(
        infile: Option<String>,
        format: Option<Format>,
        config: impl Into<Config>,
    ) -> Result<Input> {
        let infile = infile.and_then(|infile| if infile == "-" { None } else { Some(infile) });
        let format = format
            .or_else(|| infile.as_deref().and_then(Format::infer_from_href))
            .unwrap_or_default();
        let config = config.into();
        let reader = if let Some(infile) = infile {
            let (object_store, path) =
                crate::object_store::parse_href_opts(&infile, config.iter())?;
            Reader::ObjectStore { object_store, path }
        } else {
            Reader::Stdin
        };
        Ok(Input {
            format,
            reader,
            config,
        })
    }

    /// Creates a new input with the given href.
    pub fn with_href(&self, href: &str) -> Result<Input> {
        let (object_store, path) = crate::object_store::parse_href_opts(&href, self.config.iter())?;
        let reader = Reader::ObjectStore { object_store, path };
        Ok(Input {
            format: self.format,
            reader,
            config: self.config.clone(),
        })
    }

    /// Gets a STAC value from the input.
    ///
    /// Uses the infile that this input was created with, if there was one ... otherwise, gets from stdin.
    pub async fn get(&self) -> Result<Value> {
        match &self.reader {
            Reader::ObjectStore { object_store, path } => {
                let bytes = object_store.get(&path).await?.bytes().await?;
                match self.format {
                    Format::Json => serde_json::from_slice(&bytes).map_err(Error::from),
                    Format::NdJson => bytes
                        .split(|c| *c == b'\n')
                        .map(|line| serde_json::from_slice::<Item>(line).map_err(Error::from))
                        .collect::<Result<Vec<_>>>()
                        .map(ItemCollection::from)
                        .map(Value::from),
                    #[cfg(feature = "geoparquet")]
                    Format::Geoparquet => stac::geoparquet::from_reader(bytes)
                        .map(Value::from)
                        .map_err(Error::from),
                }
            }
            Reader::Stdin => match self.format {
                Format::Json => serde_json::from_reader(std::io::stdin()).map_err(Error::from),
                Format::NdJson => stac::ndjson::from_buf_reader(BufReader::new(std::io::stdin()))
                    .map(Value::from)
                    .map_err(Error::from),
                #[cfg(feature = "geoparquet")]
                Format::Geoparquet => {
                    use std::io::Read;

                    let mut buf = Vec::new();
                    let _ = std::io::stdin().read_to_end(&mut buf)?;
                    stac::geoparquet::from_reader(bytes::Bytes::from(buf))
                        .map(Value::from)
                        .map_err(Error::from)
                }
            },
        }
    }
}
