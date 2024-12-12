//! Convert between [ItemCollection] and [Table]. (experimental)
//!
//!  ⚠️ geoarrow support is currently experimental, and may break on any release.

pub mod json;

use crate::{Error, ItemCollection, Result};
use arrow_json::ReaderBuilder;
use arrow_schema::{DataType, Field, SchemaBuilder, TimeUnit};
use geo_types::Geometry;
use geoarrow::{array::GeometryBuilder, table::Table};
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
    let item_collection = item_collection.into();
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
}
