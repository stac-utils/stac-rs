//! Convert between [ItemCollection] and [Table]. (experimental)
//!
//!  ⚠️ geoarrow support is currently experimental, and may break on any release.

mod json;

use crate::{Error, FlatItem, Item, ItemCollection, Result};
use arrow_json::ReaderBuilder;
use arrow_schema::{DataType, Field, SchemaBuilder, TimeUnit};
use geo_types::Geometry;
use geoarrow::{
    array::{AsGeometryArray, MixedGeometryBuilder},
    datatypes::{Dimension, GeoDataType},
    table::Table,
    trait_::GeometryArrayAccessor,
};
use geojson::Value;
use serde_json::json;
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
pub fn to_table(item_collection: ItemCollection) -> Result<Table> {
    let mut values = Vec::with_capacity(item_collection.items.len());
    let mut builder = MixedGeometryBuilder::<i32, 2>::new();
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
        geoarrow::chunked_array::from_geoarrow_chunks(&[&array])?,
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
    use GeoDataType::*;

    let (index, _) = table
        .schema()
        .column_with_name("geometry")
        .ok_or(Error::MissingGeometry)?;
    let mut json_rows = json::record_batches_to_json_rows(table.batches(), index)?;
    let mut items = Vec::new();
    for chunk in table.geometry_column(Some(index))?.geometry_chunks() {
        for i in 0..chunk.len() {
            let value = match chunk.data_type() {
                Point(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_point_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_point_3d().value_as_geo(i)),
                },
                LineString(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_line_string_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_line_string_3d().value_as_geo(i)),
                },
                LargeLineString(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_large_line_string_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_large_line_string_3d().value_as_geo(i)),
                },
                Polygon(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_polygon_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_polygon_3d().value_as_geo(i)),
                },
                LargePolygon(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_large_polygon_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_large_polygon_3d().value_as_geo(i)),
                },
                MultiPoint(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_multi_point_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_multi_point_3d().value_as_geo(i)),
                },
                LargeMultiPoint(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_large_multi_point_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_large_multi_point_3d().value_as_geo(i)),
                },
                MultiLineString(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_multi_line_string_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_multi_line_string_3d().value_as_geo(i)),
                },
                LargeMultiLineString(_, dimension) => match dimension {
                    Dimension::XY => {
                        Value::from(&chunk.as_large_multi_line_string_2d().value_as_geo(i))
                    }
                    Dimension::XYZ => {
                        Value::from(&chunk.as_large_multi_line_string_3d().value_as_geo(i))
                    }
                },
                MultiPolygon(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_multi_polygon_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_multi_polygon_3d().value_as_geo(i)),
                },
                LargeMultiPolygon(_, dimension) => match dimension {
                    Dimension::XY => {
                        Value::from(&chunk.as_large_multi_polygon_2d().value_as_geo(i))
                    }
                    Dimension::XYZ => {
                        Value::from(&chunk.as_large_multi_polygon_3d().value_as_geo(i))
                    }
                },
                Mixed(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_mixed_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_mixed_3d().value_as_geo(i)),
                },
                LargeMixed(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_large_mixed_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_large_mixed_3d().value_as_geo(i)),
                },
                GeometryCollection(_, dimension) => match dimension {
                    Dimension::XY => {
                        Value::from(&chunk.as_geometry_collection_2d().value_as_geo(i))
                    }
                    Dimension::XYZ => {
                        Value::from(&chunk.as_geometry_collection_3d().value_as_geo(i))
                    }
                },
                LargeGeometryCollection(_, dimension) => match dimension {
                    Dimension::XY => {
                        Value::from(&chunk.as_large_geometry_collection_2d().value_as_geo(i))
                    }
                    Dimension::XYZ => {
                        Value::from(&chunk.as_large_geometry_collection_3d().value_as_geo(i))
                    }
                },
                WKB => Value::from(&chunk.as_wkb().value_as_geo(i)),
                LargeWKB => Value::from(&chunk.as_large_wkb().value_as_geo(i)),
                Rect(dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_rect_2d().value_as_geo(i)),
                    Dimension::XYZ => Value::from(&chunk.as_rect_3d().value_as_geo(i)),
                },
            };
            let mut row = json_rows
                .next()
                .expect("we shouldn't run out of rows before we're done");
            let _ = row.insert(
                "geometry".into(),
                serde_json::to_value(geojson::Geometry::new(value))?,
            );
            let flat_item: FlatItem = serde_json::from_value(serde_json::Value::Object(row))?;
            items.push(Item::try_from(flat_item)?);
        }
    }
    Ok(items.into())
}

// We only run tests when the geoparquet feature is enabled so that we don't
// have to add geoarrow as a dev dependency for all builds.
#[cfg(all(test, feature = "geoparquet"))]
mod tests {
    use crate::ItemCollection;
    use geoarrow::io::parquet::GeoParquetRecordBatchReaderBuilder;
    use std::fs::File;

    #[test]
    fn to_table() {
        let item = crate::read("examples/simple-item.json").unwrap();
        let _ = super::to_table(vec![item].into()).unwrap();
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
        let item = crate::read("examples/simple-item.json").unwrap();
        let table = super::to_table(vec![item].into()).unwrap();
        let _ = super::from_table(table).unwrap();
    }

    #[test]
    fn roundtrip_with_missing_asset() {
        let items: ItemCollection = crate::read("data/two-sentinel-2-items.json").unwrap();
        let table = super::to_table(items).unwrap();
        let _ = super::from_table(table).unwrap();
    }
}
