use crate::{Error, Href, Object};
use serde_json::Value;
use std::{fs::File, io::BufReader};
use url::Url;

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
    #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/60554
    fn read<'a, T: Into<Option<&'a str>>>(&self, href: &str, base: T) -> Result<Object, Error> {
        let href = Href::new(href, base)?;
        self.read_href(href)
    }

    /// Reads a STAC object from an href, storing that href on the object for later reference.
    ///
    /// # Examples
    ///
    /// `Reader` implements `Read`:
    ///
    /// ```
    /// use stac::{Read, Reader, Href};
    /// let href = Href::new("data/catalog.json", None).unwrap();
    /// let reader = Reader::default();
    /// let catalog = reader.read_href(href).unwrap();
    /// ```
    fn read_href(&self, href: Href) -> Result<Object, Error> {
        let value = self.read_json(&href)?;
        let mut object = Object::from_value(value)?;
        object.as_mut().href = Some(href);
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
    /// use stac::{Read, Reader, Catalog, Href};
    /// let reader = Reader::default();
    /// let href = Href::new("data/catalog.json", None).unwrap();
    /// let value = reader.read_json(&href).unwrap();
    /// ```
    fn read_json(&self, href: &Href) -> Result<Value, Error>;
}

/// The default STAC reader.
#[derive(Debug, Default)]
pub struct Reader {}

impl Read for Reader {
    fn read_json(&self, href: &Href) -> Result<Value, Error> {
        match href {
            Href::Path(path) => {
                let file = File::open(path)?;
                let buf_reader = BufReader::new(file);
                serde_json::from_reader(buf_reader).map_err(Error::from)
            }
            Href::Url(url) => read_json_from_url(url),
        }
    }
}

#[cfg(feature = "reqwest")]
fn read_json_from_url(url: &Url) -> Result<Value, Error> {
    reqwest::blocking::get(url.as_str())
        .and_then(|response| response.json())
        .map_err(Error::from)
}

#[cfg(not(feature = "reqwest"))]
fn read_json_from_url(_: &Url) -> Result<Value, Error> {
    Err(Error::ReqwestNotEnabled)
}

#[cfg(test)]
mod tests {
    use super::{Read, Reader};

    #[test]
    fn read_fs() {
        let reader = Reader::default();
        let catalog = reader.read("data/catalog.json", None).unwrap();
        assert_eq!(
            catalog.as_ref().href.as_ref().unwrap().as_path().unwrap(),
            std::fs::canonicalize("data/catalog.json").unwrap()
        );
    }

    #[cfg(feature = "reqwest")]
    #[test]
    fn read_url() {
        let reader = Reader::default();
        let _ = reader
            .read("https://planetarycomputer.microsoft.com/api/stac/v1", None)
            .unwrap();
    }

    #[cfg(not(feature = "reqwest"))]
    #[test]
    fn read_url() {
        let reader = Reader::default();
        let _ = reader
            .read("https://planetarycomputer.microsoft.com/api/stac/v1", None)
            .unwrap_err();
    }
}
