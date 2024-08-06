use crate::{Error, Result};
use std::str::FromStr;

/// The STAC output format.
#[derive(Clone, Copy, Debug, Default)]
pub enum Format {
    /// JSON (the default).
    #[default]
    Json,
}

impl FromStr for Format {
    type Err = Error;
    fn from_str(s: &str) -> Result<Format> {
        match s.to_ascii_lowercase().as_str() {
            "json" | "geojson" => Ok(Format::Json),
            _ => Err(Error::UnsupportedFormat(s.to_string())),
        }
    }
}
