//! Read and write STAC objects from the local filesystem.

use crate::{Error, Object};
use serde_json::Value;
use std::{
    convert::TryFrom,
    fs::File,
    io::{BufReader, BufWriter},
    path::Path,
};

/// Reads a STAC object from a path.
///
/// # Examples
///
/// ```
/// let catalog = stac::fs::read_from_path("data/catalog.json").unwrap();
/// assert!(catalog.is_catalog());
/// ```
pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<Object, Error> {
    let file = File::open(path)?;
    let buf_reader = BufReader::new(file);
    let value: Value = serde_json::from_reader(buf_reader)?;
    Object::try_from(value)
}

/// Writes a STAC object to a path.
///
/// # Examples
///
/// ```no_run
/// # use stac::Item;
/// let item = Item::new("an-id");
/// stac::fs::write_to_path(item, "item.json").unwrap();
/// ```
pub fn write_to_path<O: Into<Object>, P: AsRef<Path>>(object: O, path: P) -> Result<(), Error> {
    let value: Value = object.into().to_value()?;
    let file = File::create(path)?;
    let buf_writer = BufWriter::new(file);
    serde_json::to_writer(buf_writer, &value)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::Item;
    use walkdir::WalkDir;

    #[test]
    fn read_all_data() {
        for entry in WalkDir::new("data")
            .into_iter()
            .map(|result| result.unwrap())
        {
            let path = entry.path();
            if path.extension().map(|ext| ext == "json").unwrap_or(false) {
                let _ = super::read_from_path(path).unwrap();
            }
        }
    }

    #[test]
    fn roundtrip() {
        let directory = tempfile::tempdir().unwrap();
        let path = directory.path().join("item.json");
        let before = Item::new("an-id");
        super::write_to_path(before.clone(), &path).unwrap();
        let object = super::read_from_path(path).unwrap();
        let after = object.as_item().unwrap();
        assert_eq!(&before, after);
    }
}
