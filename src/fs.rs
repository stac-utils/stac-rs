//! Read and write STAC objects on the local filesystem.

use crate::{Error, Object};
use std::{fs::File, io::BufReader, path::Path};

/// Reads a STAC object from a filesystem path.
///
/// # Examples
///
/// ```
/// let catalog = stac::fs::read_from_path("data/catalog.json").unwrap();
/// ```
pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<Object, Error> {
    let file = File::open(&path)?;
    let buf_reader = BufReader::new(file);
    let value = serde_json::from_reader(buf_reader)?;
    Object::new(value, path.as_ref().to_string_lossy())
}

#[cfg(test)]
mod tests {
    #[test]
    fn read_from_path() {
        let item = super::read_from_path("data/simple-item.json").unwrap();
        assert!(item.is_item());
        assert_eq!(item.href().unwrap(), "data/simple-item.json");
    }
}
