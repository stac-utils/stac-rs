use crate::{config::Config, Error, Result};
use object_store::{local::LocalFileSystem, path::Path, ObjectStore};
use stac::{Format, Item, ItemCollection, Value};
use std::io::BufReader;
use url::Url;

/// The input to a CLI run.
#[derive(Debug, Default)]
pub(crate) struct Input {
    format: Format,
    reader: Reader,
    config: Config,
}

#[derive(Debug, Default)]
enum Reader {
    ObjectStore {
        object_store: Box<dyn ObjectStore>,
        path: Path,
    },
    #[default]
    Stdin,
}

impl Input {
    /// Creates a new input.
    pub(crate) fn new(
        infile: impl Into<Option<String>>,
        format: impl Into<Option<Format>>,
        config: impl Into<Config>,
    ) -> Result<Input> {
        let infile = infile
            .into()
            .and_then(|infile| if infile == "-" { None } else { Some(infile) });
        let format = format
            .into()
            .or_else(|| infile.as_deref().and_then(Format::infer_from_href))
            .unwrap_or_default();
        let config = config.into();
        let reader = if let Some(infile) = infile {
            let (object_store, path) = parse_href_opts(&infile, config.iter())?;
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
    pub(crate) fn with_href(&self, href: &str) -> Result<Input> {
        let (object_store, path) = parse_href_opts(href, self.config.iter())?;
        let reader = Reader::ObjectStore { object_store, path };
        Ok(Input {
            format: self.format,
            reader,
            config: self.config.clone(),
        })
    }

    /// Gets a STAC value from the input.
    pub(crate) async fn get(&self) -> Result<Value> {
        tracing::debug!("getting {}", self.format);
        match &self.reader {
            Reader::ObjectStore { object_store, path } => {
                let bytes = object_store.get(path).await?.bytes().await?;
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

pub(crate) fn parse_href_opts<I, K, V>(
    href: &str,
    options: I,
) -> Result<(Box<dyn ObjectStore>, Path)>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    if let Ok(url) = Url::parse(href) {
        object_store::parse_url_opts(&url, options).map_err(Error::from)
    } else {
        let path = Path::from_filesystem_path(href)?;
        let object_store = LocalFileSystem::new();
        Ok((Box::new(object_store), path))
    }
}
