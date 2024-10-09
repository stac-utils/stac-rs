use std::io::Read;

use crate::{options::Options, Error, Result};
use stac::{Format, Value};

/// The input to a CLI run.
#[derive(Debug, Default)]
pub(crate) struct Input {
    format: Option<Format>,
    options: Options,
    href: Option<String>,
}

impl Input {
    /// Creates a new input.
    pub(crate) fn new(
        href: impl Into<Option<String>>,
        format: impl Into<Option<Format>>,
        options: impl Into<Options>,
    ) -> Input {
        let href = href
            .into()
            .and_then(|href| if href == "-" { None } else { Some(href) });
        Input {
            format: format.into(),
            href,
            options: options.into(),
        }
    }

    /// Creates a new input with the given href.
    pub(crate) fn with_href(&self, href: impl Into<Option<String>>) -> Input {
        Input {
            format: self.format,
            href: href.into(),
            options: self.options.clone(),
        }
    }

    /// Gets a STAC value from the input.
    pub(crate) async fn get(&self) -> Result<Value> {
        if let Some(href) = self.href.as_deref() {
            self.format
                .or_else(|| Format::infer_from_href(href))
                .unwrap_or_default()
                .get_opts(href, self.options.iter())
                .await
                .map_err(Error::from)
        } else {
            let mut buf = Vec::new();
            let _ = std::io::stdin().read_to_end(&mut buf);
            self.format
                .unwrap_or_default()
                .from_bytes(buf)
                .map_err(Error::from)
        }
    }

    /// Gets a serde_json value from the input.
    pub(crate) async fn get_json(&self) -> Result<serde_json::Value> {
        if let Some(href) = self.href.as_deref() {
            let (object_store, path) =
                object_store::parse_url_opts(&href.parse()?, self.options.iter())?;
            let get_result = object_store.get(&path).await?;
            let bytes = get_result.bytes().await?;
            let format = self
                .format
                .or_else(|| Format::infer_from_href(href))
                .unwrap_or_default();
            match format {
                Format::Json(..) => serde_json::from_slice(&bytes).map_err(Error::from),
                _ => {
                    let value: Value = format.from_bytes(bytes)?;
                    serde_json::to_value(value).map_err(Error::from)
                }
            }
        } else {
            let mut buf = Vec::new();
            let _ = std::io::stdin().read_to_end(&mut buf);
            let format = self.format.unwrap_or_default();
            match format {
                Format::Json(..) => serde_json::from_slice(&buf).map_err(Error::from),
                _ => {
                    let value: Value = format.from_bytes(buf)?;
                    serde_json::to_value(value).map_err(Error::from)
                }
            }
        }
    }
}
