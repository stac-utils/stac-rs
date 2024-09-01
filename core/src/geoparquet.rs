//! Read data from and write data to [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet/blob/main/spec/stac-geoparquet-spec.md). (experimental)
//!
//!  ⚠️ geoparquet support is currently experimental, and may break on any release.

use crate::{Error, ItemCollection, Result, Value};
use geoarrow::io::parquet::GeoParquetRecordBatchReaderBuilder;
use parquet::file::reader::ChunkReader;
use std::io::Write;

/// Writes a [Value] to a [std::io::Write] as
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
/// stac::geoparquet::to_writer(&mut cursor, item.into()).unwrap();
/// ```
pub fn to_writer<W>(writer: W, value: Value) -> Result<()>
where
    W: Write + Send,
{
    match value {
        Value::ItemCollection(item_collection) => {
            let table = crate::geoarrow::to_table(item_collection)?;
            geoarrow::io::parquet::write_geoparquet(
                table.into_record_batch_reader(),
                writer,
                &Default::default(),
            )
            .map_err(Error::from)
        }
        Value::Item(item) => to_writer(writer, ItemCollection::from(vec![item.clone()]).into()),
        _ => Err(Error::IncorrectType {
            actual: value.type_name().to_string(),
            expected: "Item or ItemCollection".to_string(),
        }),
    }
}

/// Reads a [ItemCollection] from a [std::io::Read] as
/// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet).
///
/// # Examples
///
/// ```
/// use std::fs::File;
///
/// let file = File::open("examples/extended-item.parquet").unwrap();
/// let item_collection = stac::geoparquet::from_reader(file).unwrap();
/// ```
pub fn from_reader<R>(reader: R) -> Result<ItemCollection>
where
    R: ChunkReader + 'static,
{
    let reader = GeoParquetRecordBatchReaderBuilder::try_new(reader)?.build()?;
    let table = reader.read_table()?;
    crate::geoarrow::from_table(table).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use crate::{Href, Item, ItemCollection};
    use bytes::Bytes;
    use std::{fs::File, io::Cursor};

    #[test]
    fn to_writer_catalog() {
        let mut cursor = Cursor::new(Vec::new());
        let catalog = crate::read("data/catalog.json").unwrap();
        let _ = super::to_writer(&mut cursor, catalog).unwrap_err();
    }

    #[test]
    fn to_writer_collection() {
        let mut cursor = Cursor::new(Vec::new());
        let collection = crate::read("data/collection.json").unwrap();
        let _ = super::to_writer(&mut cursor, collection).unwrap_err();
    }

    #[test]
    fn to_writer_item_collection() {
        let mut cursor = Cursor::new(Vec::new());
        let item = crate::read("data/simple-item.json").unwrap();
        let item_collection = ItemCollection::from(vec![item]);
        super::to_writer(&mut cursor, item_collection.into()).unwrap();
    }

    #[test]
    fn to_writer_item() {
        let mut cursor = Cursor::new(Vec::new());
        let item = crate::read("data/simple-item.json").unwrap();
        super::to_writer(&mut cursor, item).unwrap();
    }

    #[test]
    fn from_reader() {
        let file = File::open("examples/extended-item.parquet").unwrap();
        let item_collection = super::from_reader(file).unwrap();
        assert_eq!(item_collection.items.len(), 1);
    }

    #[test]
    fn roundtrip() {
        let mut item: Item = crate::read("data/simple-item.json").unwrap();
        item.clear_href();
        let mut cursor = Cursor::new(Vec::new());
        super::to_writer(&mut cursor, item.clone().into()).unwrap();
        let bytes = Bytes::from(cursor.into_inner());
        let item_collection = super::from_reader(bytes).unwrap();
        assert_eq!(item_collection.items[0], item);
    }
}
