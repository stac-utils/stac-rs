use crate::Error;
use std::{path::Path, str::FromStr};

/// Formats that can be used for STAC data.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub enum Format {
    /// JSON format, the default.
    #[default]
    Json,

    /// [Geoparquet](https://github.com/stac-utils/stac-geoparquet)
    #[cfg(feature = "parquet")]
    GeoParquet,
}

impl Format {
    /// Returns the correct format for this href's extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_cli::Format;
    ///
    /// assert_eq!(Format::Json, Format::from_href("item.json").unwrap());
    /// #[cfg(feature = "parquet")]
    /// assert_eq!(Format::GeoParquet, Format::from_href("items.parquet").unwrap());
    /// ```
    pub fn from_href(href: &str) -> Option<Format> {
        Path::new(href)
            .extension()
            .and_then(|e| e.to_str())
            .and_then(|e| match e {
                "json" => Some(Format::Json),
                #[cfg(feature = "parquet")]
                "parquet" | "geoparquet" => Some(Format::GeoParquet),
                _ => None,
            })
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "json" => Ok(Format::Json),
            #[cfg(feature = "parquet")]
            "geoparquet" | "parquet" => Ok(Format::GeoParquet),
            _ => Err(Error::InvalidFormat(s.to_string())),
        }
    }
}
