//! Convert between [ItemCollection] and [Table]. (experimental)
//!
//!  ⚠️ geoarrow support is currently experimental, and may break on any release.

pub mod json;

use crate::{Error, ItemCollection, Result};
use arrow_array::{cast::AsArray, types::GenericBinaryType, GenericByteArray, RecordBatch};
use arrow_json::ReaderBuilder;
use arrow_schema::{DataType, Field, SchemaBuilder, TimeUnit};
use geo_types::Geometry;
use geoarrow::{
    array::{CoordType, GeometryBuilder, NativeArrayDyn, WKBArray},
    datatypes::NativeType,
    table::Table,
    ArrayBase,
};
use serde_json::{json, Value};
use std::sync::Arc;

const DATETIME_COLUMNS: [&str; 8] = [
    "datetime",
    "start_datetime",
    "end_datetime",
    "created",
    "updated",
    "expires",
    "published",
    "unpublished",
];

const GEOMETRY_COLUMNS: [&str; 2] = ["geometry", "proj:geometry"];

/// Converts an [ItemCollection] to a [Table].
///
/// Any invalid attributes in the items (e.g. top-level attributes that conflict
/// with STAC spec attributes) will be dropped with a warning.
///
/// # Examples
///
/// ```
/// use stac::ItemCollection;
///
/// let item = stac::read("examples/simple-item.json").unwrap();
/// let item_collection = ItemCollection::from(vec![item]);
/// let table = stac::geoarrow::to_table(item_collection).unwrap();
/// ```
pub fn to_table(item_collection: impl Into<ItemCollection>) -> Result<Table> {
    let item_collection: ItemCollection = item_collection.into();
    let mut values = Vec::with_capacity(item_collection.items.len());
    let mut builder = GeometryBuilder::new();
    for mut item in item_collection.items {
        builder.push_geometry(
            item.geometry
                .take()
                .and_then(|geometry| Geometry::try_from(geometry).ok())
                .as_ref(),
        )?;
        let flat_item = item.into_flat_item(true)?;
        let mut value = serde_json::to_value(flat_item)?;
        {
            let value = value
                .as_object_mut()
                .expect("a flat item should serialize to an object");
            let _ = value.remove("geometry");
            if let Some(bbox) = value.remove("bbox") {
                let bbox = bbox
                    .as_array()
                    .expect("STAC items should always have a list as their bbox");
                if bbox.len() == 4 {
                    let _ = value.insert("bbox".into(), json!({
                        "xmin": bbox[0].as_number().expect("all bbox values should be a number"),
                        "ymin": bbox[1].as_number().expect("all bbox values should be a number"),
                        "xmax": bbox[2].as_number().expect("all bbox values should be a number"),
                        "ymax": bbox[3].as_number().expect("all bbox values should be a number"),
                    }));
                } else if bbox.len() == 6 {
                    let _ = value.insert("bbox".into(), json!({
                        "xmin": bbox[0].as_number().expect("all bbox values should be a number"),
                        "ymin": bbox[1].as_number().expect("all bbox values should be a number"),
                        "zmin": bbox[2].as_number().expect("all bbox values should be a number"),
                        "xmax": bbox[3].as_number().expect("all bbox values should be a number"),
                        "ymax": bbox[4].as_number().expect("all bbox values should be a number"),
                        "zmax": bbox[5].as_number().expect("all bbox values should be a number"),
                    }));
                } else {
                    return Err(Error::InvalidBbox(
                        bbox.iter().filter_map(|v| v.as_f64()).collect(),
                    ));
                }
            }
        }
        values.push(value);
    }
    let schema = arrow_json::reader::infer_json_schema_from_iterator(values.iter().map(Ok))?;
    let mut schema_builder = SchemaBuilder::new();
    for field in schema.fields().iter() {
        if DATETIME_COLUMNS.contains(&field.name().as_str()) {
            schema_builder.push(Field::new(
                field.name(),
                DataType::Timestamp(TimeUnit::Millisecond, Some("UTC".into())),
                field.is_nullable(),
            ));
        } else {
            schema_builder.push(field.clone());
        }
    }
    let metadata = schema.metadata;
    let schema = Arc::new(schema_builder.finish().with_metadata(metadata));
    let mut decoder = ReaderBuilder::new(schema.clone()).build_decoder()?;
    decoder.serialize(&values)?;
    let batch = decoder.flush()?.ok_or(Error::NoItems)?;
    let array = builder.finish();
    Table::from_arrow_and_geometry(
        vec![batch],
        schema,
        geoarrow::chunked_array::ChunkedNativeArrayDyn::from_geoarrow_chunks(&[&array])?
            .into_inner(),
    )
    .map_err(Error::from)
}

/// Converts a [Table] to an [ItemCollection].
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "geoparquet")]
/// # {
/// use std::fs::File;
/// use geoarrow::io::parquet::GeoParquetRecordBatchReaderBuilder;
///
/// let file = File::open("data/extended-item.parquet").unwrap();
/// let reader = GeoParquetRecordBatchReaderBuilder::try_new(file)
///     .unwrap()
///     .build()
///     .unwrap();
/// let table = reader.read_table().unwrap();
/// let item_collection = stac::geoarrow::from_table(table).unwrap();
/// # }
/// ```
pub fn from_table(table: Table) -> Result<ItemCollection> {
    json::from_table(table)?
        .into_iter()
        .map(|item| serde_json::from_value(Value::Object(item)).map_err(Error::from))
        .collect::<Result<Vec<_>>>()
        .map(ItemCollection::from)
}

/// Converts a geometry column to geoarrow native type.
pub fn with_native_geometries(mut record_batch: RecordBatch) -> Result<RecordBatch> {
    for column_name in GEOMETRY_COLUMNS {
        if let Some((index, _)) = record_batch.schema().column_with_name(column_name) {
            let geometry_column = record_batch.remove_column(index);
            let binary_array: GenericByteArray<GenericBinaryType<i32>> =
                geometry_column.as_binary::<i32>().clone();
            let wkb_array = WKBArray::new(binary_array, Default::default());
            let geometry_array = geoarrow::io::wkb::from_wkb(
                &wkb_array,
                NativeType::Geometry(CoordType::Interleaved),
                false,
            )?;
            let mut columns = record_batch.columns().to_vec();
            let mut schema_builder = SchemaBuilder::from(&*record_batch.schema());
            schema_builder.push(geometry_array.extension_field());
            let schema = schema_builder.finish();
            columns.push(geometry_array.to_array_ref());
            record_batch = RecordBatch::try_new(schema.into(), columns)?;
        }
    }
    Ok(record_batch)
}

/// Converts a geometry column to geoarrow.wkb.
pub fn with_wkb_geometries(mut record_batch: RecordBatch) -> Result<RecordBatch> {
    for column_name in GEOMETRY_COLUMNS {
        if let Some((index, field)) = record_batch.schema().column_with_name(column_name) {
            let geometry_column = record_batch.remove_column(index);
            let wkb_array = geoarrow::io::wkb::to_wkb::<i32>(&NativeArrayDyn::from_arrow_array(
                &geometry_column,
                field,
            )?);
            let mut columns = record_batch.columns().to_vec();
            let mut schema_builder = SchemaBuilder::from(&*record_batch.schema());
            schema_builder.push(wkb_array.extension_field());
            let schema = schema_builder.finish();
            columns.push(wkb_array.to_array_ref());
            record_batch = RecordBatch::try_new(schema.into(), columns)?;
        }
    }
    Ok(record_batch)
}

/// Adds geoarrow wkb metadata to a geometry column.
pub fn add_wkb_metadata(mut record_batch: RecordBatch) -> Result<RecordBatch> {
    for column_name in GEOMETRY_COLUMNS {
        if let Some((index, field)) = record_batch.schema().column_with_name(column_name) {
            let mut metadata = field.metadata().clone();
            let _ = metadata.insert(
                "ARROW:extension:name".to_string(),
                "geoarrow.wkb".to_string(),
            );
            let field = field.clone().with_metadata(metadata);
            let mut schema_builder = SchemaBuilder::from(&*record_batch.schema());
            let field_ref = schema_builder.field_mut(index);
            *field_ref = field.into();
            let schema = schema_builder.finish();
            record_batch = record_batch.with_schema(schema.into())?;
        }
    }
    Ok(record_batch)
}

// We only run tests when the geoparquet feature is enabled so that we don't
// have to add geoarrow as a dev dependency for all builds.
#[cfg(all(test, feature = "geoparquet"))]
mod tests {
    use crate::{Item, ItemCollection};
    use geoarrow::io::parquet::GeoParquetRecordBatchReaderBuilder;
    use std::fs::File;

    #[test]
    fn to_table() {
        let item: Item = crate::read("examples/simple-item.json").unwrap();
        let _ = super::to_table(vec![item]).unwrap();
    }

    #[test]
    fn from_table() {
        let file = File::open("data/extended-item.parquet").unwrap();
        let reader = GeoParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();
        let table = reader.read_table().unwrap();
        let item_collection = super::from_table(table).unwrap();
        assert_eq!(item_collection.items.len(), 1);
    }

    #[test]
    fn roundtrip() {
        let item: Item = crate::read("examples/simple-item.json").unwrap();
        let table = super::to_table(vec![item]).unwrap();
        let _ = super::from_table(table).unwrap();
    }

    #[test]
    fn roundtrip_with_missing_asset() {
        let items: ItemCollection = crate::read("data/two-sentinel-2-items.json").unwrap();
        let table = super::to_table(items).unwrap();
        let _ = super::from_table(table).unwrap();
    }

    #[test]
    fn with_wkb_geometry() {
        let item: Item = crate::read("examples/simple-item.json").unwrap();
        let table = super::to_table(vec![item]).unwrap();
        let (mut record_batches, _) = table.into_inner();
        assert_eq!(record_batches.len(), 1);
        let record_batch = record_batches.pop().unwrap();
        let _ = super::with_wkb_geometries(record_batch).unwrap();
    }

    #[test]
    fn with_wkb_geometries() {
        let item: Item = crate::read("examples/proj-geometry.json").unwrap();
        let table = super::to_table(vec![item]).unwrap();
        let (mut record_batches, _) = table.into_inner();
        assert_eq!(record_batches.len(), 1);
        let record_batch = record_batches.pop().unwrap();
        let _ = super::with_wkb_geometries(record_batch).unwrap();
    }
}
