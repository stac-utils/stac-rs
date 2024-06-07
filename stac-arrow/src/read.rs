use crate::{Error, Result};
use arrow::array::{AsArray, OffsetSizeTrait, RecordBatch};
use geo::Geometry;
use geoarrow::table::GeoTable;
use geozero::wkb::{FromWkb, WkbDialect};
use serde_json::{Map, Value};
use stac::{item::GeoparquetItem, Item};
use std::io::Cursor;

/// Converts a [GeoTable] into a vector of [Items](Item).
///
/// # Examples
///
/// ```
/// use std::fs::File;
///
/// let file = File::open("data/naip.parquet").unwrap();
/// let geo_table = geoarrow::io::parquet::read_geoparquet(file, Default::default()).unwrap();
/// let items = stac_arrow::geo_table_to_items(geo_table).unwrap();
/// assert_eq!(items.len(), 5);
/// ```
pub fn geo_table_to_items(geo_table: GeoTable) -> Result<Vec<Item>> {
    Reader::new().read::<i32>(geo_table)
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

    /// Reads a [GeoTable] into a vector of items.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::fs::File;
    /// use stac_arrow::Reader;
    ///
    /// let file = File::open("data/naip.parquet").unwrap();
    /// let geo_table = geoarrow::io::parquet::read_geoparquet(file, Default::default()).unwrap();
    /// let reader = Reader::new();
    /// let items = reader.read::<i32>(geo_table).unwrap();
    /// assert_eq!(items.len(), 5);
    /// ```
    pub fn read<O: OffsetSizeTrait>(&self, geo_table: GeoTable) -> Result<Vec<Item>> {
        let mut items = Vec::with_capacity(geo_table.len());
        let (_, record_batches, _) = geo_table.into_inner();
        for record_batch in record_batches {
            items.extend(self.record_batch_to_items::<O>(record_batch)?);
        }
        Ok(items)
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
    ///     items.extend(reader.record_batch_to_items::<i32>(result.unwrap()).unwrap());
    /// }
    /// assert_eq!(items.len(), 5);
    /// ```
    #[allow(deprecated)] // We find that `record_batches_to_json_rows` is faster than serializing-then-deserializing with `Writer`
    pub fn record_batch_to_items<O: OffsetSizeTrait>(
        &self,
        mut record_batch: RecordBatch,
    ) -> Result<Vec<Item>> {
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
    use super::Reader;
    use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;
    use stac_validate::Validate;
    use std::fs::File;

    #[test]
    fn record_batch_to_items() {
        let file = File::open("data/naip.parquet").unwrap();
        let mut parquet_reader = ParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();
        let reader = Reader::new();
        let items = parquet_reader
            .next()
            .map(|result| {
                reader
                    .record_batch_to_items::<i32>(result.unwrap())
                    .unwrap()
            })
            .unwrap();
        assert_eq!(items.len(), 5);
        for item in items {
            assert_eq!(item.extensions.len(), 6);
            assert!(item.geometry.is_some());
            assert!(item.bbox.is_some());
            assert!(!item.links.is_empty());
            assert!(!item.assets.is_empty());
            assert!(item.collection.is_some());
            item.validate().unwrap();
        }
    }

    #[test]
    fn geo_table_to_items() {
        let file = File::open("data/naip.parquet").unwrap();
        let geo_table = geoarrow::io::parquet::read_geoparquet(file, Default::default()).unwrap();
        let items = super::geo_table_to_items(geo_table).unwrap();
        assert_eq!(items.len(), 5);
    }
}
