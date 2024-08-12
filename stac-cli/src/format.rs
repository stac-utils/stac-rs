use crate::{Error, Result};
use serde::de::DeserializeOwned;
use std::{io::Read, str::FromStr};

/// The STAC output format.
#[derive(Clone, Copy, Debug, Default)]
pub enum Format {
    /// stac-geparquet
    Geoparquet,

    /// JSON (the default).
    #[default]
    Json,
}

impl FromStr for Format {
    type Err = Error;
    fn from_str(s: &str) -> Result<Format> {
        match s.to_ascii_lowercase().as_str() {
            "json" | "geojson" => Ok(Format::Json),
            "parquet" | "geoparquet" => Ok(Format::Geoparquet),
            _ => Err(Error::UnsupportedFormat(s.to_string())),
        }
    }
}

impl Format {
    pub(crate) fn maybe_from_href(href: &str) -> Option<Format> {
        href.rsplit_once('.')
            .and_then(|(_, ext)| Format::from_str(ext).ok())
    }

    pub(crate) async fn read_href<D: DeserializeOwned>(&self, href: Option<&str>) -> Result<D> {
        if let Some(href) = href.and_then(|href| if href == "-" { None } else { Some(href) }) {
            match *self {
                Format::Geoparquet => {
                    unimplemented!("waiting on geoarrow v0.3 release");
                    // let item_collection = if let Some(url) = stac::href_to_url(href) {
                    // stac_geoparquet::from_reader(reqwest::blocking::get(url)?.bytes()?)?
                    // } else {
                    // let file = File::open(href)?;
                    // stac_geoparquet::from_reader(file)?
                    // };
                    // serde_json::from_value(serde_json::to_value(item_collection)?)
                    //     .map_err(Error::from)
                }
                Format::Json => stac_async::read_json(href).await.map_err(Error::from),
            }
        } else {
            match *self {
                Format::Geoparquet => {
                    let mut buf = Vec::new();
                    let _ = std::io::stdin().read_to_end(&mut buf)?;
                    unimplemented!("waiting on geoarrow v0.3 release");
                    // let item_collection = stac_geoparquet::from_reader(Bytes::from(buf))?;
                    // serde_json::from_value(serde_json::to_value(item_collection)?)
                    //     .map_err(Error::from)
                }
                Format::Json => serde_json::from_reader(std::io::stdin()).map_err(Error::from),
            }
        }
    }
}
