use crate::{Error, FromJson, Item, ItemCollection, Result, SelfHref, Value};
use bytes::Bytes;
use serde::Serialize;
use std::{
    fs::File,
    io::{BufRead, BufReader, BufWriter, Write},
    path::Path,
};

/// Create a STAC object from newline-delimited JSON.
pub trait FromNdjson: FromJson {
    /// Reads newline-delimited JSON data from a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{FromNdjson, ItemCollection};
    ///
    /// let item_collection = ItemCollection::from_ndjson_path("data/items.ndjson").unwrap();
    /// ```
    fn from_ndjson_path(path: impl AsRef<Path>) -> Result<Self> {
        Self::from_json_path(path)
    }

    /// Creates a STAC object from ndjson bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Read};
    /// use stac::{ItemCollection, FromNdjson};
    ///
    /// let mut buf = Vec::new();
    /// File::open("data/items.ndjson").unwrap().read_to_end(&mut buf).unwrap();
    /// let item_collection = ItemCollection::from_ndjson_bytes(buf).unwrap();
    /// ```
    fn from_ndjson_bytes(bytes: impl Into<Bytes>) -> Result<Self> {
        let bytes = bytes.into();
        Self::from_json_slice(&bytes)
    }
}

/// Write a STAC object to newline-delimited JSON.
pub trait ToNdjson: Serialize {
    /// Writes a value to a path as newline-delimited JSON.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::{ToNdjson, ItemCollection, Item};
    ///
    /// let item_collection: ItemCollection = vec![Item::new("a"), Item::new("b")].into();
    /// item_collection.to_ndjson_path("items.ndjson").unwrap();
    /// ```
    fn to_ndjson_path(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::create(path)?;
        self.to_ndjson_writer(file)
    }

    /// Writes a value to a writer as newline-delimited JSON.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::{ToNdjson, ItemCollection, Item};
    ///
    /// let item_collection: ItemCollection = vec![Item::new("a"), Item::new("b")].into();
    /// let mut buf = Vec::new();
    /// item_collection.to_ndjson_writer(&mut buf).unwrap();
    /// ```
    fn to_ndjson_writer(&self, writer: impl Write) -> Result<()> {
        serde_json::to_writer(writer, self).map_err(Error::from)
    }

    /// Writes a value as newline-delimited JSON bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{ToNdjson, Item, ItemCollection};
    ///
    /// let item_collection: ItemCollection = vec![Item::new("a"), Item::new("b")].into();
    /// let bytes = item_collection.to_ndjson_vec().unwrap();
    /// ```
    fn to_ndjson_vec(&self) -> Result<Vec<u8>> {
        serde_json::to_vec(self).map_err(Error::from)
    }
}

impl FromNdjson for Item {}
impl FromNdjson for crate::Catalog {}
impl FromNdjson for crate::Collection {}
impl FromNdjson for ItemCollection {
    fn from_ndjson_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let reader = BufReader::new(File::open(path)?);
        let mut items = Vec::new();
        for line in reader.lines() {
            items.push(serde_json::from_str(&line?)?);
        }
        let mut item_collection = ItemCollection::from(items);
        *item_collection.self_href_mut() = Some(path.into());
        Ok(item_collection)
    }
    fn from_ndjson_bytes(bytes: impl Into<Bytes>) -> Result<Self> {
        bytes
            .into()
            .split(|b| *b == b'\n')
            .filter_map(|line| {
                if line.is_empty() {
                    None
                } else {
                    Some(serde_json::from_slice::<Item>(line).map_err(Error::from))
                }
            })
            .collect::<Result<Vec<_>>>()
            .map(ItemCollection::from)
    }
}
impl FromNdjson for Value {
    fn from_ndjson_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let reader = BufReader::new(File::open(path)?);
        let mut values: Vec<Value> = Vec::new();
        for line in reader.lines() {
            values.push(serde_json::from_str(&line?)?);
        }
        vec_into_value(values)
    }
    fn from_ndjson_bytes(bytes: impl Into<Bytes>) -> Result<Self> {
        let values = bytes
            .into()
            .split(|b| *b == b'\n')
            .filter_map(|line| {
                if line.is_empty() {
                    None
                } else {
                    Some(serde_json::from_slice::<Value>(line).map_err(Error::from))
                }
            })
            .collect::<Result<Vec<_>>>()?;
        vec_into_value(values)
    }
}

fn vec_into_value(mut values: Vec<Value>) -> Result<Value> {
    if values.len() == 1 {
        Ok(values.pop().unwrap())
    } else {
        Ok(ItemCollection::from(
            values
                .into_iter()
                .map(Item::try_from)
                .collect::<Result<Vec<_>>>()?,
        )
        .into())
    }
}

impl ToNdjson for Item {}
impl ToNdjson for crate::Catalog {}
impl ToNdjson for crate::Collection {}

impl ToNdjson for ItemCollection {
    fn to_ndjson_path(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = BufWriter::new(File::create(path)?);
        self.to_ndjson_writer(file)
    }

    fn to_ndjson_writer(&self, mut writer: impl Write) -> Result<()> {
        for item in &self.items {
            serde_json::to_writer(&mut writer, item)?;
            writeln!(&mut writer)?;
        }
        Ok(())
    }

    fn to_ndjson_vec(&self) -> Result<Vec<u8>> {
        let mut vec = Vec::new();
        self.to_ndjson_writer(&mut vec)?;
        Ok(vec)
    }
}

impl ToNdjson for Value {
    fn to_ndjson_path(&self, path: impl AsRef<Path>) -> Result<()> {
        match self {
            Value::Item(item) => item.to_ndjson_path(path),
            Value::Catalog(catalog) => catalog.to_ndjson_path(path),
            Value::Collection(collection) => collection.to_ndjson_path(path),
            Value::ItemCollection(item_collection) => item_collection.to_ndjson_path(path),
        }
    }

    fn to_ndjson_writer(&self, writer: impl Write) -> Result<()> {
        match self {
            Value::Item(item) => item.to_ndjson_writer(writer),
            Value::Catalog(catalog) => catalog.to_ndjson_writer(writer),
            Value::Collection(collection) => collection.to_ndjson_writer(writer),
            Value::ItemCollection(item_collection) => item_collection.to_ndjson_writer(writer),
        }
    }

    fn to_ndjson_vec(&self) -> Result<Vec<u8>> {
        match self {
            Value::Item(item) => item.to_ndjson_vec(),
            Value::Catalog(catalog) => catalog.to_ndjson_vec(),
            Value::Collection(collection) => collection.to_ndjson_vec(),
            Value::ItemCollection(item_collection) => item_collection.to_ndjson_vec(),
        }
    }
}

impl ToNdjson for serde_json::Value {
    fn to_ndjson_path(&self, path: impl AsRef<Path>) -> Result<()> {
        let file = File::create(path)?;
        self.to_ndjson_writer(file)
    }

    fn to_ndjson_vec(&self) -> Result<Vec<u8>> {
        let mut buf = Vec::new();
        self.to_ndjson_writer(&mut buf)?;
        Ok(buf)
    }

    fn to_ndjson_writer(&self, mut writer: impl Write) -> Result<()> {
        if let serde_json::Value::Array(array) = self {
            for value in array {
                serde_json::to_writer(&mut writer, value)?;
                writeln!(&mut writer)?;
            }
        } else if let Some(features) = self
            .as_object()
            .and_then(|o| o.get("features"))
            .and_then(|f| f.as_array())
        {
            for value in features {
                serde_json::to_writer(&mut writer, value)?;
                writeln!(&mut writer)?;
            }
        } else {
            serde_json::to_writer(&mut writer, self)?;
            writeln!(&mut writer)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::FromNdjson;
    use crate::{ItemCollection, SelfHref, Value};
    use std::{fs::File, io::Read};

    #[test]
    fn item_collection_read() {
        let item_collection = ItemCollection::from_ndjson_path("data/items.ndjson").unwrap();
        assert_eq!(item_collection.items.len(), 2);
        assert!(item_collection
            .self_href()
            .unwrap()
            .as_str()
            .ends_with("data/items.ndjson"));
    }

    #[test]
    fn item_collection_from_bytes() {
        let mut buf = Vec::new();
        let _ = File::open("data/items.ndjson")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let item_collection = ItemCollection::from_ndjson_bytes(buf).unwrap();
        assert_eq!(item_collection.items.len(), 2);
    }

    #[test]
    fn value_read() {
        let _ = Value::from_ndjson_path("data/items.ndjson").unwrap();
    }

    #[test]
    fn value_from_bytes() {
        let mut buf = Vec::new();
        let _ = File::open("data/items.ndjson")
            .unwrap()
            .read_to_end(&mut buf)
            .unwrap();
        let _ = Value::from_ndjson_bytes(buf).unwrap();
    }
}
