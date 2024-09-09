//! Input and output (IO) functions for newline-delimited JSON data.

use crate::{io::Read, ItemCollection, Result};
use serde::Serialize;
use std::{
    fs::File,
    io::{BufRead, BufReader, Write},
};

/// Reads an item collection from a JSON href as newline-delimited JSON.
///
/// # Examples
///
/// ```
/// let item_collection = stac::ndjson::read("data/items.ndjson").unwrap();
/// ```
pub fn read(href: impl ToString) -> Result<ItemCollection> {
    NdJsonReader::read(href)
}

/// Reads newline-delimited JSON to an item collection.
///
/// # Examples
///
/// ```
/// use std::{io::BufReader, fs::File};
///
/// let file = File::open("data/items.ndjson").unwrap();
/// let item_collection = stac::ndjson::from_buf_reader(BufReader::new(file)).unwrap();
/// ```
pub fn from_buf_reader(reader: impl BufRead) -> Result<ItemCollection> {
    let mut items = Vec::new();
    for result in reader.lines() {
        items.push(serde_json::from_str(&result?)?);
    }
    Ok(items.into())
}

/// Writes a newline-delimited JSON iterable of values.
///
/// # Examples
///
/// ```
/// use stac::Item;
///
/// let item = Item::new("an-id");
/// let mut bytes = Vec::new();
/// stac::ndjson::to_writer(&mut bytes, vec![item].iter());
/// ```
pub fn to_writer<T>(mut writer: impl Write, items: impl Iterator<Item = T>) -> Result<()>
where
    T: Serialize,
{
    for item in items {
        serde_json::to_writer(&mut writer, &item)?;
        writeln!(&mut writer)?;
    }
    Ok(())
}

struct NdJsonReader;

impl Read<ItemCollection> for NdJsonReader {
    fn read_from_file(file: File) -> Result<ItemCollection> {
        from_buf_reader(BufReader::new(file))
    }

    #[cfg(feature = "reqwest")]
    fn from_response(response: reqwest::blocking::Response) -> Result<ItemCollection> {
        from_buf_reader(BufReader::new(response))
    }
}
