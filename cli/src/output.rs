//! Structures for writing output data.

use crate::{options::Options, value::Value, Error, Result};
use object_store::{local::LocalFileSystem, ObjectStore, PutResult};
use stac::{io::Config, Format};
use std::{io::IsTerminal, path::Path, pin::Pin};
use tokio::{
    fs::File,
    io::{AsyncWrite, AsyncWriteExt},
};
use url::Url;

/// The output from a CLI run.
pub(crate) struct Output {
    pub(crate) format: Format,
    href: Option<String>,
    stream: Pin<Box<dyn AsyncWrite + Send>>,
    config: Config,
}

impl Output {
    /// Creates a new output from an optional outfile and an optional format.
    pub(crate) async fn new(
        href: Option<String>,
        format: Option<Format>,
        options: impl Into<Options>,
        create_parent_directories: bool,
    ) -> Result<Output> {
        let mut format = format
            .or_else(|| href.as_deref().and_then(Format::infer_from_href))
            .unwrap_or_default();
        let config = Config::new().format(Some(format)).options(options.into());
        let stream = if let Some(href) = href.as_deref() {
            if let Ok(url) = Url::parse(href) {
                if url.scheme() == "file" {
                    create_file_stream(
                        url.to_file_path().unwrap_or_default(),
                        create_parent_directories,
                    )
                    .await?
                } else {
                    Box::pin(config.buf_writer(&url)?)
                }
            } else {
                create_file_stream(href, create_parent_directories).await?
            }
        } else {
            if std::io::stdout().is_terminal() {
                format = format.pretty();
            }
            Box::pin(tokio::io::stdout())
        };
        Ok(Output {
            href,
            format,
            stream,
            config,
        })
    }

    /// Streams a value to the output
    pub(crate) async fn stream(&mut self, value: Value) -> Result<()> {
        let bytes = value.into_ndjson()?;
        self.stream.write_all(&bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Puts a value to the output.
    pub(crate) async fn put(&mut self, value: Value) -> Result<Option<PutResult>> {
        let bytes = match value {
            Value::Json(value) => self.format.json_to_vec(value)?,
            Value::Stac(value) => self.format.value_to_vec(value)?,
        };
        if let Some(href) = self.href.as_deref() {
            let (object_store, path): (Box<dyn ObjectStore>, _) = match Url::parse(href) {
                Ok(url) => self.config.object_store(&url)?,
                Err(_) => {
                    let path = object_store::path::Path::from_filesystem_path(href)?;
                    (Box::new(LocalFileSystem::new()), path)
                }
            };
            object_store
                .put(&path, bytes.into())
                .await
                .map(Some)
                .map_err(Error::from)
        } else {
            self.stream.write_all(&bytes).await?;
            self.stream.flush().await?;
            Ok(None)
        }
    }
}

async fn create_file_stream(
    path: impl AsRef<Path>,
    create_parent_directories: bool,
) -> Result<Pin<Box<dyn AsyncWrite + Send>>> {
    let path = path.as_ref();
    if create_parent_directories {
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
    }
    let file = File::create(path).await?;
    Ok(Box::pin(file))
}
