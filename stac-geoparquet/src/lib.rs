use parquet::file::reader::ChunkReader;
use stac::{ItemCollection, Value};
use std::io::Write;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum Error {
    /// [geoarrow::error::GeoArrowError]
    #[error(transparent)]
    GeoArrow(#[from] geoarrow::error::GeoArrowError),

    /// [stac_arrow::Error]
    #[error(transparent)]
    StacArrow(#[from] stac_arrow::Error),

    /// This STAC type is not supported by stac-geoparquet.
    #[error("unsupported type: {0}")]
    UnsupportedType(&'static str),
}

pub type Result<T> = std::result::Result<T, Error>;

/// Writes a [stac::Value] to a [std::io::Write] as
/// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet).
///
/// Currently, will throw an error if the value is not an item or an item
/// collection.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
/// use stac::Item;
///
/// let item: Item = stac::read("data/simple-item.json").unwrap();
/// let mut cursor = Cursor::new(Vec::new());
/// stac_geoparquet::to_writer(&mut cursor, item.into()).unwrap();
/// ```
pub fn to_writer<W>(writer: W, value: Value) -> Result<()>
where
    W: Write + Send,
{
    match value {
        Value::ItemCollection(item_collection) => {
            let mut table = stac_arrow::to_table(item_collection)?;
            geoarrow::io::parquet::write_geoparquet(&mut table, writer, &Default::default())
                .map_err(Error::from)
        }
        Value::Item(item) => to_writer(writer, ItemCollection::from(vec![item.clone()]).into()),
        _ => Err(Error::UnsupportedType(value.type_name())),
    }
}

/// Reads a [stac::ItemCollection] from a [std::io::Read] as
/// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet).
///
/// # Examples
///
/// ```
/// use std::fs::File;
///
/// let file = File::open("examples/extended-item.parquet").unwrap();
/// let item_collection = stac_geoparquet::from_reader(file).unwrap();
/// ```
pub fn from_reader<R>(reader: R) -> Result<ItemCollection>
where
    R: ChunkReader + 'static,
{
    let table = geoarrow::io::parquet::read_geoparquet(reader, Default::default())?;
    stac_arrow::from_table(table).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use stac::ItemCollection;
    use std::{fs::File, io::Cursor};

    #[test]
    fn to_writer_catalog() {
        let mut cursor = Cursor::new(Vec::new());
        let catalog = stac::read("data/catalog.json").unwrap();
        super::to_writer(&mut cursor, catalog).unwrap_err();
    }

    #[test]
    fn to_writer_collection() {
        let mut cursor = Cursor::new(Vec::new());
        let collection = stac::read("data/collection.json").unwrap();
        super::to_writer(&mut cursor, collection).unwrap_err();
    }

    #[test]
    fn to_writer_item_collection() {
        let mut cursor = Cursor::new(Vec::new());
        let item = stac::read("data/simple-item.json").unwrap();
        let item_collection = ItemCollection::from(vec![item]);
        super::to_writer(&mut cursor, item_collection.into()).unwrap();
    }

    #[test]
    fn to_writer_item() {
        let mut cursor = Cursor::new(Vec::new());
        let item = stac::read("data/simple-item.json").unwrap();
        super::to_writer(&mut cursor, item).unwrap();
    }

    #[test]
    fn from_reader() {
        let file = File::open("examples/extended-item.parquet").unwrap();
        let item_collection = super::from_reader(file).unwrap();
        assert_eq!(item_collection.items.len(), 1);
    }
}

// From https://github.com/rust-lang/cargo/issues/383#issuecomment-720873790,
// may they be forever blessed.
#[cfg(doctest)]
mod readme {
    macro_rules! external_doc_test {
        ($x:expr) => {
            #[doc = $x]
            extern "C" {}
        };
    }

    external_doc_test!(include_str!("../README.md"));
}
