use crate::{Error, Object};
use serde_json::Value;
use std::{fs::File, io::BufReader};

/// A trait derived by all STAC readers.
pub trait Read {
    /// Reads a STAC object from an href and, optionally, a base (e.g. a parent object).
    ///
    /// # Examples
    ///
    /// `Reader` implements `Read`:
    ///
    /// ```
    /// use stac::{Read, Reader, Catalog};
    /// let reader = Reader::default();
    /// let catalog = reader.read("data/catalog.json", None).unwrap();
    /// ```
    fn read(&self, href: &str, base: Option<&str>) -> Result<Object, Error> {
        let href = crate::utils::absolute_href(href, base)?;
        let value = self.read_json(&href)?;
        let mut object = Object::from_value(value)?;
        object.as_mut().href = Some(href.to_string());
        Ok(object)
    }

    /// Reads JSON data from an HREF.
    ///
    /// Generally shouldn't be used -- prefer [Read::read] to get STAC objects.
    ///
    /// # Examples
    ///
    /// `Reader` implements `Read`:
    ///
    /// ```
    /// use stac::{Read, Reader, Catalog};
    /// let reader = Reader::default();
    /// let value = reader.read_json("data/catalog.json").unwrap();
    /// ```
    fn read_json(&self, href: &str) -> Result<Value, Error>;
}

/// The default STAC reader.
#[derive(Debug, Default)]
pub struct Reader {}

impl Read for Reader {
    fn read_json(&self, href: &str) -> Result<Value, Error> {
        let file = File::open(&href)?;
        let buf_reader = BufReader::new(file);
        serde_json::from_reader(buf_reader).map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::{Read, Reader};

    #[test]
    fn read_fs() {
        let reader = Reader::default();
        let catalog = reader.read("data/catalog.json", None).unwrap();
        assert_eq!(
            catalog.as_ref().href.as_deref().unwrap(),
            std::fs::canonicalize("data/catalog.json")
                .unwrap()
                .to_str()
                .unwrap()
        );
    }
}
