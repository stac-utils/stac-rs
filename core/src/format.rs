use crate::{Error, Result};
use std::{fmt::Display, str::FromStr};

/// The format of STAC data.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Format {
    /// JSON data (the default).
    #[default]
    Json,

    /// Newline-delimited JSON.
    NdJson,

    /// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet)
    #[cfg(feature = "geoparquet")]
    Geoparquet,
}

impl Format {
    /// Infer a format from a file extension.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Format;
    ///
    /// assert_eq!(Format::Json, Format::infer_from_href("item.json").unwrap());
    /// ```
    pub fn infer_from_href(href: &str) -> Option<Format> {
        href.rsplit_once('.').and_then(|(_, ext)| ext.parse().ok())
    }
}

impl Display for Format {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::Json => "json",
            Self::NdJson => "ndjson",
            #[cfg(feature = "geoparquet")]
            Self::Geoparquet => "geoparquet",
        })
    }
}

impl FromStr for Format {
    type Err = Error;

    fn from_str(s: &str) -> Result<Format> {
        match s.to_ascii_lowercase().as_str() {
            "json" | "geojson" => Ok(Self::Json),
            "ndjson" => Ok(Self::NdJson),
            #[cfg(feature = "geoparquet")]
            "geoparquet" | "parquet" => Ok(Self::Geoparquet),
            _ => Err(Error::UnsupportedFormat(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Format;

    #[test]
    #[cfg(feature = "geoparquet")]
    fn parse_geoparquet() {
        assert_eq!("parquet".parse::<Format>().unwrap(), Format::Geoparquet);
    }
}
