use crate::{Error, Href, Object};
use serde_json::Value;
use std::{fs::File, io::BufReader};
use url::Url;

/// A trait derived by all STAC readers.
pub trait Read {
    /// Reads a STAC object from an href.
    ///
    /// # Examples
    ///
    /// `Reader` implements `Read`:
    ///
    /// ```
    /// use stac::{Read, Reader, Catalog};
    /// let reader = Reader::default();
    /// let catalog = reader.read("data/catalog.json").unwrap();
    /// ```
    fn read<T, E>(&self, href: T) -> Result<Object, Error>
    where
        T: TryInto<Href, Error = E>,
        Error: From<E>,
    {
        let href = href.try_into()?;
        let value = self.read_json(&href)?;
        let mut object = Object::from_value(value)?;
        object.href = Some(href);
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
    /// let href = Href::new("data/catalog.json").unwrap();
    /// let value = reader.read_json(&href).unwrap();
    /// ```
    fn read_json(&self, href: &Href) -> Result<Value, Error>;
}

/// The default STAC reader.
#[derive(Debug, Default)]
pub struct Reader {}

impl Read for Reader {
    fn read_json(&self, href: &Href) -> Result<Value, Error> {
        if let Some(path) = href.to_path() {
            let file = File::open(path)?;
            let buf_reader = BufReader::new(file);
            serde_json::from_reader(buf_reader).map_err(Error::from)
        } else {
            // FIXME this smells bad
            read_json_from_url(
                href.as_url()
                    .expect("if the href is not a path it should be a url"),
            )
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
    use std::path::Path;

    #[test]
    fn read_fs() {
        let reader = Reader::default();
        let catalog = reader.read("data/catalog.json").unwrap();
        assert_eq!(
            catalog.href.as_ref().unwrap().to_path().unwrap(),
            Path::new("data/catalog.json"),
        );
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
