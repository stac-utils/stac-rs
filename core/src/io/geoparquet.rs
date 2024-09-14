//! Read data from and write data to [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet/blob/main/spec/stac-geoparquet-spec.md) (experimental).
//!
//!  ⚠️ geoparquet support is currently experimental, and may break on any release.

use crate::{Error, ItemCollection, Result};
use geoarrow::io::parquet::{GeoParquetRecordBatchReaderBuilder, GeoParquetWriterOptions};
use parquet::{
    basic::Compression,
    file::{properties::WriterProperties, reader::ChunkReader},
};
use std::io::Write;

/// Reads a [ItemCollection] from a [ChunkReader] as
/// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet).
///
/// # Examples
///
/// ```
/// use std::fs::File;
///
/// let file = File::open("data/extended-item.parquet").unwrap();
/// let item_collection = stac::io::geoparquet::from_reader(file).unwrap();
/// ```
pub fn from_reader<R>(reader: R) -> Result<ItemCollection>
where
    R: ChunkReader + 'static,
{
    let reader = GeoParquetRecordBatchReaderBuilder::try_new(reader)?.build()?;
    let table = reader.read_table()?;
    crate::io::geoarrow::from_table(table).map_err(Error::from)
}

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
/// let item: Item = stac::read("examples/simple-item.json").unwrap();
/// let mut cursor = Cursor::new(Vec::new());
/// stac::io::geoparquet::to_writer(&mut cursor, item).unwrap();
/// ```
pub fn to_writer<W>(writer: W, item_collection: ItemCollection) -> Result<()>
where
    W: Write + Send,
{
    to_writer_with_options(writer, item_collection, &Default::default())
}

/// Writes a [Value] to a [std::io::Write] as
/// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet) with the provided compression.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
/// use stac::Item;
/// use parquet::basic::Compression;
///
/// let item: Item = stac::read("examples/simple-item.json").unwrap();
/// let mut cursor = Cursor::new(Vec::new());
/// stac::io::geoparquet::to_writer_with_compression(&mut cursor, item, Compression::SNAPPY).unwrap();
/// ```
pub fn to_writer_with_compression<W>(
    writer: W,
    item_collection: ItemCollection,
    compression: Compression,
) -> Result<()>
where
    W: Write + Send,
{
    let mut options = GeoParquetWriterOptions::default();
    let writer_properties = WriterProperties::builder()
        .set_compression(compression)
        .build();
    options.writer_properties = Some(writer_properties);
    to_writer_with_options(writer, item_collection, &options)
}

/// Writes a [Value] to a [std::io::Write] as
/// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet) with the provided options.
///
/// # Examples
///
/// ```
/// use std::io::Cursor;
/// use stac::Item;
/// use parquet::basic::Compression;
///
/// let item: Item = stac::read("examples/simple-item.json").unwrap();
/// let mut cursor = Cursor::new(Vec::new());
/// stac::io::geoparquet::to_writer_with_options(&mut cursor, vec![item].into(), &Default::default()).unwrap();
/// ```
pub fn to_writer_with_options<W>(
    writer: W,
    item_collection: ItemCollection,
    options: &GeoParquetWriterOptions,
) -> Result<()>
where
    W: Write + Send,
{
    let table = crate::io::geoarrow::to_table(item_collection)?;
    geoarrow::io::parquet::write_geoparquet(table.into_record_batch_reader(), writer, options)
        .map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use crate::{Item, ItemCollection, Object};
    use bytes::Bytes;
    use std::{fs::File, io::Cursor};

    #[test]
    fn to_writer_item_collection() {
        let mut cursor = Cursor::new(Vec::new());
        let item = crate::read("examples/simple-item.json").unwrap();
        let item_collection = ItemCollection::from(vec![item]);
        super::to_writer(&mut cursor, item_collection).unwrap();
    }

    #[test]
    fn from_reader() {
        let file = File::open("data/extended-item.parquet").unwrap();
        let item_collection = super::from_reader(file).unwrap();
        assert_eq!(item_collection.items.len(), 1);
    }

    #[test]
    fn roundtrip() {
        let mut item: Item = crate::read("examples/simple-item.json").unwrap();
        *item.href_mut() = None;
        let mut cursor = Cursor::new(Vec::new());
        super::to_writer(&mut cursor, vec![item.clone()].into()).unwrap();
        let bytes = Bytes::from(cursor.into_inner());
        let item_collection = super::from_reader(bytes).unwrap();
        assert_eq!(item_collection.items[0], item);
    }
}
