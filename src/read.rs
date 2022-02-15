use crate::{Error, Object, PathBufHref};
use serde_json::Value;
use std::{fs::File, io::BufReader};
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
    fn read<T>(&self, href: T) -> Result<Object, Error>
    where
        T: Into<PathBufHref>,
    {
        let href = href.into();
        let value = self.read_json(href.clone())?;
        let mut object = Object::from_value(value)?;
        object.href = Some(href.into());
        Ok(object)
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
    /// let value = reader.read_json("data/catalog.json").unwrap();
    /// assert_eq!(value.get("type").unwrap().as_str().unwrap(), "Catalog");
    /// ```
    fn read_json<T: Into<PathBufHref>>(&self, href: T) -> Result<Value, Error>;
}

/// A basic reader for STAC objects.
///
/// This reader uses the standard library to read from the filesystem. If the
/// `reqwest` feature is enabled (it is by default), blocking
/// [reqwest](https://docs.rs/reqwest/latest/reqwest/) calls are used to read
/// from urls. In the future, async calls may be supported, but are not yet.
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
    fn read_json<T: Into<PathBufHref>>(&self, href: T) -> Result<Value, Error> {
        match href.into() {
            PathBufHref::Path(path) => {
                let file = File::open(path)?;
                let buf_reader = BufReader::new(file);
                serde_json::from_reader(buf_reader).map_err(Error::from)
            }
            PathBufHref::Url(url) => read_json_from_url(&url),
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
        let catalog = reader.read("data/catalog.json").unwrap();
        assert_eq!(catalog.href.as_ref().unwrap().as_str(), "data/catalog.json",);
    }

    #[cfg(feature = "reqwest")]
    #[test]
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