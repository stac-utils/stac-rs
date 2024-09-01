//! Input and output (IO) functions for geoparquet data.

use super::Read;
use crate::{ItemCollection, Result};
#[cfg(feature = "reqwest")]
use reqwest::blocking::Response;
use std::fs::File;

/// Reads an [ItemCollection] from a geoparquet href.
///
/// # Examples
///
/// ```
/// let item_collection = stac::io::geoparquet::read("data/extended-item.parquet").unwrap();
/// ```
pub fn read(href: impl ToString) -> Result<ItemCollection> {
    GeoparquetReader::read(href)
}

struct GeoparquetReader;

impl Read<ItemCollection> for GeoparquetReader {
    fn read_from_file(file: File) -> Result<ItemCollection> {
        crate::geoparquet::from_reader(file)
    }
    #[cfg(feature = "reqwest")]
    fn from_response(response: Response) -> Result<ItemCollection> {
        crate::geoparquet::from_reader(response.bytes()?)
    }
}
