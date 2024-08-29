use crate::{Error, Result};
use bytes::Bytes;
use clap::ValueEnum;
use serde::de::DeserializeOwned;
use std::{fs::File, io::Read};

/// The STAC output format.
#[derive(Clone, Copy, Debug, Default, ValueEnum)]
pub enum Format {
    /// stac-geoparquet
    Parquet,

    /// JSON (the default).
    #[default]
    Json,
}

impl Format {
    pub(crate) fn maybe_from_href(href: &str) -> Option<Format> {
        href.rsplit_once('.')
            .and_then(|(_, ext)| Format::from_str(ext, true).ok())
    }

    pub(crate) async fn read_href<D: DeserializeOwned>(&self, href: Option<&str>) -> Result<D> {
        if let Some(href) = href.and_then(|href| if href == "-" { None } else { Some(href) }) {
            match *self {
                Format::Parquet => {
                    let item_collection = if let Some(url) = stac::href_to_url(href) {
                        stac_geoparquet::from_reader(reqwest::blocking::get(url)?.bytes()?)?
                    } else {
                        let file = File::open(href)?;
                        stac_geoparquet::from_reader(file)?
                    };
                    serde_json::from_value(serde_json::to_value(item_collection)?)
                        .map_err(Error::from)
                }
                Format::Json => stac_async::read_json(href).await.map_err(Error::from),
            }
        } else {
            match *self {
                Format::Parquet => {
                    let mut buf = Vec::new();
                    let _ = std::io::stdin().read_to_end(&mut buf)?;
                    let item_collection = stac_geoparquet::from_reader(Bytes::from(buf))?;
                    serde_json::from_value(serde_json::to_value(item_collection)?)
                        .map_err(Error::from)
                }
                Format::Json => serde_json::from_reader(std::io::stdin()).map_err(Error::from),
            }
        }
    }
}
