//! Input and output (IO) functions for JSON data.

use super::Read;
use crate::{Error, Href, Result};
#[cfg(feature = "reqwest")]
use reqwest::blocking::Response;
use serde::de::DeserializeOwned;
use std::{fs::File, io::BufReader};

/// Reads any STAC value from a JSON href.
///
/// # Examples
///
/// ```
/// let item: stac::Item = stac::io::json::read("examples/simple-item.json").unwrap();
/// ```
pub fn read<T: Href + DeserializeOwned>(href: impl ToString) -> Result<T> {
    JsonReader::read(href)
}

struct JsonReader;

impl<T: Href + DeserializeOwned> Read<T> for JsonReader {
    fn read_from_file(file: File) -> Result<T> {
        serde_json::from_reader(BufReader::new(file)).map_err(Error::from)
    }
    #[cfg(feature = "reqwest")]
    fn from_response(response: Response) -> Result<T> {
        response.json().map_err(Error::from)
    }
}
