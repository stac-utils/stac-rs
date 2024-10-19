use crate::{Error, Href, Result};
use serde::{de::DeserializeOwned, Serialize};
use std::{
    fs::File,
    io::{Read, Write},
    path::Path,
};

/// Create a STAC object from JSON.
pub trait FromJson: DeserializeOwned + Href {
    /// Reads JSON data from a file.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{FromJson, Item};
    ///
    /// let item = Item::from_json_path("examples/simple-item.json").unwrap();
    /// ```
    fn from_json_path(path: impl AsRef<Path>) -> Result<Self> {
        let path = path.as_ref();
        let mut buf = Vec::new();
        let _ = File::open(path)?.read_to_end(&mut buf)?;
        let mut value = Self::from_json_slice(&buf)?;
        value.set_href(path.to_string_lossy());
        Ok(value)
    }

    /// Creates an object from JSON bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::{fs::File, io::Read};
    /// use stac::{Item, FromJson};
    ///
    /// let mut buf = Vec::new();
    /// File::open("examples/simple-item.json").unwrap().read_to_end(&mut buf).unwrap();
    /// let item = Item::from_json_slice(&buf).unwrap();
    /// ```
    fn from_json_slice(slice: &[u8]) -> Result<Self> {
        serde_json::from_slice(slice).map_err(Error::from)
    }
}

/// Write a STAC object to JSON.
pub trait ToJson: Serialize {
    /// Writes a value to a path as JSON.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::{ToJson, Item};
    ///
    /// Item::new("an-id").to_json_path("an-id.json", true).unwrap();
    /// ```
    fn to_json_path(&self, path: impl AsRef<Path>, pretty: bool) -> Result<()> {
        let file = File::create(path)?;
        self.to_json_writer(file, pretty)
    }

    /// Writes a value as JSON.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{ToJson, Item};
    ///
    /// let mut buf = Vec::new();
    /// Item::new("an-id").to_json_writer(&mut buf, true).unwrap();
    /// ```
    fn to_json_writer(&self, writer: impl Write, pretty: bool) -> Result<()> {
        if pretty {
            serde_json::to_writer_pretty(writer, self).map_err(Error::from)
        } else {
            serde_json::to_writer(writer, self).map_err(Error::from)
        }
    }

    /// Writes a value as JSON bytes.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{ToJson, Item};
    ///
    /// Item::new("an-id").to_json_vec(true).unwrap();
    /// ```
    fn to_json_vec(&self, pretty: bool) -> Result<Vec<u8>> {
        if pretty {
            serde_json::to_vec_pretty(self).map_err(Error::from)
        } else {
            serde_json::to_vec(self).map_err(Error::from)
        }
    }
}

impl<T: DeserializeOwned + Href> FromJson for T {}
impl<T: Serialize> ToJson for T {}

#[cfg(test)]
mod tests {
    use super::FromJson;
    use crate::{Href, Item};

    #[test]
    fn set_href() {
        let item = Item::from_json_path("examples/simple-item.json").unwrap();
        assert!(item.href().unwrap().ends_with("examples/simple-item.json"));
    }
}
