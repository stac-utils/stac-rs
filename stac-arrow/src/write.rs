use crate::{Error, Result};
use arrow::{
    array::{Float64Builder, RecordBatch, StructBuilder, TimestampMicrosecondBuilder},
    compute::kernels::cast_utils::Parser,
    datatypes::{ArrowPrimitiveType, DataType, Field, SchemaBuilder, TimestampMicrosecondType},
};
use arrow_json::ReaderBuilder;
use geo::Geometry;
use geoarrow::{array::MixedGeometryBuilder, table::GeoTable, GeometryArrayTrait};
use stac::Item;
use std::{collections::HashMap, sync::Arc};

const DEFAULT_BATCH_SIZE: usize = 1000;
const DATETIME_ATTRIBUTES: [&str; 8] = [
    "datetime",
    "start_datetime",
    "end_datetime",
    "created",
    "updated",
    "expires",
    "published",
    "unpublished",
];

/// Converts items to a [GeoTable].
///
/// # Examples
///
/// ```
/// use stac::ItemCollection;
///
/// let item_collection: ItemCollection = stac::read_json("data/naip.json").unwrap();
/// let geo_table = stac_arrow::items_to_geo_table(item_collection.items).unwrap();
/// ```
pub fn items_to_geo_table(items: Vec<Item>) -> Result<GeoTable> {
    Writer::new().write(items)
}

/// A structure for writing items to record batches and geo tables.
#[derive(Debug)]
pub struct Writer {
    batch_size: usize,
}

impl Writer {
    /// Creates a new writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_arrow::Writer;
    ///
    /// let writer = Writer::new();
    /// ```
    pub fn new() -> Writer {
        Writer::default()
    }

    /// Sets the batch size for this writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_arrow::Writer;
    ///
    /// let writer = Writer::new().batch_size(42);
    /// ```
    pub fn batch_size(mut self, batch_size: usize) -> Writer {
        self.batch_size = batch_size;
        self
    }

    /// Writes a vector of items to a [GeoTable].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::ItemCollection;
    /// use stac_arrow::Writer;
    ///
    /// let item_collection: ItemCollection = stac::read_json("data/naip.json").unwrap();
    /// let writer = Writer::new();
    /// let geo_table = writer.write(item_collection.items).unwrap();
    /// ```
    pub fn write(&self, items: Vec<Item>) -> Result<GeoTable> {
        let mut record_batches: Vec<RecordBatch> = Vec::new();
        for chunk in items.chunks(self.batch_size) {
            let record_batch = self.items_to_record_batch(chunk.to_vec())?;
            if let Some(first) = record_batches.first() {
                if first.schema() != record_batch.schema() {
                    return Err(Error::DifferentSchemas(
                        (*first.schema()).clone(),
                        (*record_batch.schema()).clone(),
                    ));
                }
            }
            record_batches.push(record_batch);
        }
        if record_batches.is_empty() {
            return Err(Error::NoItems);
        }
        let (geometry_column_index, _) = record_batches[0]
            .schema()
            .column_with_name("geometry")
            .expect("should have a geometry field");
        GeoTable::try_new(
            record_batches[0].schema(),
            record_batches,
            geometry_column_index,
        )
        .map_err(Error::from)
    }

    /// Converts items to a record batch.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::ItemCollection;
    /// use stac_arrow::Writer;
    ///
    /// let item_collection: ItemCollection = stac::read_json("data/naip.json").unwrap();
    /// let writer = Writer::new();
    /// let record_batch = writer.items_to_record_batch(item_collection.items).unwrap();
    /// ```
    pub fn items_to_record_batch(&self, items: Vec<Item>) -> Result<RecordBatch> {
        if items.is_empty() {
            return Err(Error::NoItems);
        }
        let mut values = Vec::with_capacity(items.len());
        let mut geometry_builder = MixedGeometryBuilder::<i32>::new();
        let mut datetime_builders = HashMap::new();
        let mut datetime_keys = Vec::new();
        // TODO support 3D bboxes
        let bbox_fields = vec![
            Field::new("xmin", DataType::Float64, false),
            Field::new("ymin", DataType::Float64, false),
            Field::new("xmax", DataType::Float64, false),
            Field::new("ymax", DataType::Float64, false),
        ];
        let mut bbox_builder = StructBuilder::from_fields(bbox_fields.clone(), items.len());
        for mut item in items {
            // TODO allow configuring dropping of invalid attributes.
            let geometry: Option<Geometry> =
                item.geometry.take().map(|g| g.try_into()).transpose()?;
            geometry_builder.push_geometry(geometry.as_ref())?;
            let geoparquet_item = item.into_geoparquet_item(true)?;
            if geoparquet_item.bbox.len() != 4 {
                return Err(Error::InvalidBbox(geoparquet_item.bbox));
            } else {
                for i in 0..4 {
                    bbox_builder
                        .field_builder::<Float64Builder>(i)
                        .unwrap()
                        .append_value(geoparquet_item.bbox[i]);
                }
                bbox_builder.append(true);
            }

            let mut value = serde_json::to_value(geoparquet_item)?;
            let _ = value
                .as_object_mut()
                .expect("geoparquet item should be a map")
                .remove("geometry");
            let _ = value
                .as_object_mut()
                .expect("geoparquet item should be a map")
                .remove("bbox");
            for key in DATETIME_ATTRIBUTES {
                let entry = datetime_builders
                    .entry(key)
                    .or_insert_with(TimestampMicrosecondBuilder::new);
                if let Some(s) = value.as_object_mut().unwrap().remove(key) {
                    if !datetime_keys.contains(&key) {
                        datetime_keys.push(key);
                    }
                    entry.append_value(
                        s.as_str()
                            .and_then(TimestampMicrosecondType::parse)
                            .ok_or_else(|| Error::InvalidDatetime(s.to_string()))?,
                    );
                } else {
                    entry.append_null();
                }
            }
            values.push(value);
        }
        let geometry = geometry_builder.finish();
        // TODO allow configuration of how many items to iterate
        let schema = arrow_json::reader::infer_json_schema_from_iterator(values.iter().map(Ok))?;
        let mut decoder = ReaderBuilder::new(Arc::new(schema.clone())).build_decoder()?;
        decoder.serialize(&values)?;
        let record_batch = decoder.flush().map(|record_batch| record_batch.unwrap())?;
        let mut builder = SchemaBuilder::from(schema.fields);
        builder.push(geometry.extension_field());
        builder.push(Field::new(
            "bbox",
            DataType::Struct(bbox_fields.into()),
            false,
        ));
        for key in &datetime_keys {
            builder.push(Field::new(*key, TimestampMicrosecondType::DATA_TYPE, true));
        }
        let schema = builder.finish();
        let mut columns = record_batch.columns().to_vec();
        columns.push(geometry.to_array_ref());
        columns.push(Arc::new(bbox_builder.finish()));
        for key in datetime_keys {
            columns.push(Arc::new(
                datetime_builders
                    .get_mut(key)
                    .expect("should be a builder for every key")
                    .finish(),
            ));
        }
        RecordBatch::try_new(Arc::new(schema), columns).map_err(Error::from)
    }
}

impl Default for Writer {
    fn default() -> Writer {
        Writer {
            batch_size: DEFAULT_BATCH_SIZE,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Writer;
    use stac::ItemCollection;

    #[test]
    fn items_to_record_batch() {
        let items: ItemCollection = stac::read_json("data/naip.json").unwrap();
        let writer = Writer::new();
        let record_batch = writer.items_to_record_batch(items.items).unwrap();
        assert_eq!(record_batch.num_rows(), 5);
    }

    #[test]
    fn items_to_geo_table() {
        let items: ItemCollection = stac::read_json("data/naip.json").unwrap();
        let geo_table = super::items_to_geo_table(items.items).unwrap();
        assert_eq!(geo_table.len(), 5);
    }
}
