//! Structures for writing output data.

use crate::{options::Options, value::Value, Error, Result};
use object_store::PutResult;
use stac::{Format, ToNdjson};
use std::{path::Path, pin::Pin};
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
    options: Options,
}

impl Output {
    /// Creates a new output from an optional outfile and an optional format.
    pub(crate) async fn new(
        href: Option<String>,
        format: Option<Format>,
        options: impl Into<Options>,
        create_parent_directories: bool,
    ) -> Result<Output> {
        let format = format
            .or_else(|| href.as_deref().and_then(Format::infer_from_href))
            .unwrap_or_default();
        let stream = if let Some(href) = href.as_deref() {
            if let Ok(url) = Url::parse(href) {
                if url.scheme() == "file" {
                    create_file_stream(
                        url.to_file_path().unwrap_or_default(),
                        create_parent_directories,
                    )
                    .await?
                } else {
                    unimplemented!("streaming to object stores is not supported");
                    // FIXME turn this into an actual error
                }
            } else {
                create_file_stream(href, create_parent_directories).await?
            }
        } else {
            Box::pin(tokio::io::stdout())
        };
        Ok(Output {
            href,
            format,
            stream,
            options: options.into(),
        })
    }

    /// Streams a value to the output
    pub(crate) async fn stream(&mut self, value: Value) -> Result<()> {
        let bytes = value.to_ndjson_vec()?;
        self.stream.write_all(&bytes).await?;
        self.stream.flush().await?;
        Ok(())
    }

    /// Puts a value to the output.
    pub(crate) async fn put(&mut self, value: Value) -> Result<Option<PutResult>> {
        if let Some(href) = self.href.as_deref() {
            self.format
                .put_opts(href, value, self.options.iter())
                .await
                .map_err(Error::from)
        } else {
            let bytes = self.format.into_vec(value)?;
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
