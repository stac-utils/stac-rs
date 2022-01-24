use crate::{Error, Object};
use std::{fs::File, io::BufReader};

/// A structure for reading STAC objects.
#[derive(Debug, Default)]
pub struct Reader {}

impl Reader {
    /// Creates a new default reader.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Reader;
    /// let reader = Reader::new();
    /// ```
    pub fn new() -> Reader {
        Reader {}
    }

    /// Reads a STAC object from an href and, optionally, an HREF Core.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Reader, Catalog};
    /// let reader = Reader::new();
    /// let catalog = reader.read("data/catalog.json", None).unwrap();
    /// ```
    pub fn read(&self, href: &str, base: Option<&str>) -> Result<Object, Error> {
        let href = crate::utils::absolute_href(href, base)?;
        let file = File::open(&href)?;
        let buf_reader = BufReader::new(file);
        let mut object = Object::from_reader(buf_reader)?;
        object.as_mut().href = Some(href.to_string());
        Ok(object)
    }
}

#[cfg(test)]
mod tests {
    use super::Reader;

    #[test]
    fn read_fs() {
        let reader = Reader::new();
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
