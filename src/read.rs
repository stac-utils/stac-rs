use crate::{Error, Href, HrefObject, Object, Result};
use path_slash::PathBufExt;
use serde_json::Value;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};
use url::Url;

/// Read STAC objects from hrefs.
///
/// # Examples
///
/// [Reader] implements `Read`:
///
/// ```
/// use stac::{Read, Reader};
/// let reader = Reader::default();
/// let object = reader.read("data/catalog.json").unwrap();
/// ```
pub trait Read {
    /// Reads a STAC object from an href.
    ///
    /// # Examples
    ///
    /// `Reader` implements `Read`:
    ///
    /// ```
    /// use stac::{Read, Reader};
    /// let reader = Reader::default();
    /// let catalog = reader.read("data/catalog.json").unwrap();
    /// ```
    fn read(&self, href: impl Into<Href>) -> Result<HrefObject> {
        let href = href.into();
        let value = self.read_json(&href)?;
        let object = Object::from_value(value)?;
        Ok(HrefObject::new(object, href))
    }

    /// Reads an [Item](crate::Href), [Catalog](crate::Href), or [Collection](crate::Href) from an [Href](crate::Href).
    ///
    /// # Examples
    ///
    /// [Reader] implements `Read`:
    ///
    /// ```
    /// use stac::{Read, Reader, Catalog, Href};
    /// let reader = Reader::default();
    /// let catalog: Catalog = reader.read_object(&Href::new("data/catalog.json")).unwrap();
    /// ```
    fn read_object<O>(&self, href: &Href) -> Result<O>
    where
        O: TryFrom<Object, Error = Error>,
    {
        let value = self.read_json(href)?;
        let object = Object::from_value(value)?;
        object.try_into()
    }

    /// Reads JSON data from an href.
    ///
    /// # Examples
    ///
    /// `Reader` implements `Read`:
    ///
    /// ```
    /// use stac::{Read, Reader, Catalog, Href};
    /// let reader = Reader::default();
    /// let href = Href::from("data/catalog.json");
    /// let value = reader.read_json(&href.into()).unwrap();
    /// assert_eq!(value.get("type").unwrap().as_str().unwrap(), "Catalog");
    /// ```
    fn read_json(&self, href: &Href) -> Result<Value> {
        match href {
            Href::Url(url) => self.read_json_from_url(url),
            Href::Path(path) => self.read_json_from_path(PathBuf::from_slash(path)),
        }
    }

    /// Reads JSON data from a [Url].
    fn read_json_from_url(&self, url: &Url) -> Result<Value>;

    /// Reads JSON data from a [Path].
    fn read_json_from_path(&self, path: impl AsRef<Path>) -> Result<Value>;
}

/// A basic reader for STAC objects.
///
/// This reader uses the standard library to read from the filesystem. If the
/// `reqwest` feature is enabled, blocking
/// [reqwest](https://docs.rs/reqwest/latest/reqwest/) calls are used to read
/// from urls.
///
/// # Examples
///
/// ```
/// use stac::{Read, Reader};
/// let reader = Reader::default();
/// let object = reader.read("data/catalog.json").unwrap();
/// ```
#[derive(Debug, Default)]
pub struct Reader();

impl Read for Reader {
    #[cfg(feature = "reqwest")]
    fn read_json_from_url(&self, url: &Url) -> Result<Value> {
        reqwest::blocking::get(url.as_str())
            .and_then(|response| response.json())
            .map_err(Error::from)
    }

    #[cfg(not(feature = "reqwest"))]
    fn read_json_from_url(&self, _: &Url) -> Result<Value> {
        Err(Error::ReqwestNotEnabled)
    }

    fn read_json_from_path(&self, path: impl AsRef<Path>) -> Result<Value> {
        let file = File::open(path)?;
        let reader = BufReader::new(file);
        serde_json::from_reader(reader).map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::{Read, Reader};

    #[test]
    fn read_fs() {
        let reader = Reader::default();
        let catalog = reader.read("data/catalog.json").unwrap();
        assert_eq!(catalog.href.as_str(), "data/catalog.json");
    }

    #[cfg(feature = "reqwest")]
    #[test]
    #[ignore]
    fn read_url() {
        let reader = Reader::default();
        let _ = reader
            .read("https://planetarycomputer.microsoft.com/api/stac/v1")
            .unwrap();
    }

    #[cfg(not(feature = "reqwest"))]
    #[test]
    fn read_url() {
        let reader = Reader::default();
        let _ = reader
            .read("https://planetarycomputer.microsoft.com/api/stac/v1")
            .unwrap_err();
    }
}
