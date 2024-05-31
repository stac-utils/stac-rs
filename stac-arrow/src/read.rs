use crate::{Error, Result};
use arrow::array::{AsArray, OffsetSizeTrait, RecordBatch};
use geo::Geometry;
use geozero::wkb::{FromWkb, WkbDialect};
use serde_json::{Map, Value};
use stac::{item::GeoparquetItem, Item};
use std::io::Cursor;

/// Converts a [RecordBatch] into a vector of [Items](Item).
///
/// # Examples
///
/// ```
/// use std::fs::File;
/// use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
///
/// let file = File::open("data/naip.parquet").unwrap();
/// let reader = ParquetRecordBatchReaderBuilder::try_new(file)
///     .unwrap()
///     .build()
///     .unwrap();
/// let mut items = Vec::new();
/// for result in reader {
///     items.extend(stac_arrow::record_batch_to_items(result.unwrap()).unwrap());
/// }
/// assert_eq!(items.len(), 5);
/// ```
pub fn record_batch_to_items(record_batch: RecordBatch) -> Result<Vec<Item>> {
    Reader::new().read::<i32>(record_batch)
}

/// Reads record batches into items.
#[derive(Debug, Default)]
pub struct Reader {}

impl Reader {
    /// Creates a new reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_arrow::Reader;
    ///
    /// let reader = Reader::new();
    /// ```
    pub fn new() -> Reader {
        Reader {}
    }

    /// Reads items from a record batch.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    /// use stac_arrow::Reader;
    ///
    /// let file = File::open("data/naip.parquet").unwrap();
    /// let parquet_reader = ParquetRecordBatchReaderBuilder::try_new(file)
    ///     .unwrap()
    ///     .build()
    ///     .unwrap();
    /// let mut items = Vec::new();
    /// let reader = Reader::new();
    /// for result in parquet_reader {
    ///     items.extend(reader.read::<i32>(result.unwrap()).unwrap());
    /// }
    /// assert_eq!(items.len(), 5);
    /// ```
    #[allow(deprecated)] // We find that `record_batches_to_json_rows` is faster than serializing-then-deserializing with `Writer`
    pub fn read<O: OffsetSizeTrait>(&self, mut record_batch: RecordBatch) -> Result<Vec<Item>> {
        let index = record_batch.schema().index_of("geometry")?;
        let geometry = record_batch.remove_column(index);
        let geometry = geometry
            .as_binary_opt::<O>()
            .ok_or_else(|| Error::NonBinaryGeometryColumn)?;
        let items: Vec<Map<String, Value>> =
            arrow_json::writer::record_batches_to_json_rows(&[&record_batch])?;
        items
            .into_iter()
            .enumerate()
            .map(|(i, item)| {
                let geoparquet_item: GeoparquetItem = serde_json::from_value(Value::Object(item))?;
                // TODO handle null geometries
                let mut item: Item = geoparquet_item.try_into()?;
                item.geometry = Some(
                    (&Geometry::from_wkb(&mut Cursor::new(geometry.value(i)), WkbDialect::Wkb)?)
                        .into(),
                );
                Ok(item)
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use stac_validate::Validate;
    use std::fs::File;

    #[test]
    fn record_batch_to_items() {
        let file = File::open("data/naip.parquet").unwrap();
        let mut reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();
        let items = reader
            .next()
            .map(|result| super::record_batch_to_items(result.unwrap()).unwrap())
            .unwrap();
        assert_eq!(items.len(), 5);
        for item in items {
            assert_eq!(item.extensions.len(), 2);
            assert!(item.geometry.is_some());
            assert!(item.bbox.is_some());
            assert!(!item.links.is_empty());
            assert!(!item.assets.is_empty());
            assert!(item.collection.is_some());
            item.validate().unwrap();
        }
    }
}
