//! Convert between [ItemCollection] and [Table].

#![deny(
    elided_lifetimes_in_paths,
    explicit_outlives_requirements,
    keyword_idents,
    macro_use_extern_crate,
    meta_variable_misuse,
    missing_abi,
    missing_debug_implementations,
    missing_docs,
    non_ascii_idents,
    noop_method_call,
    rust_2021_incompatible_closure_captures,
    rust_2021_incompatible_or_patterns,
    rust_2021_prefixes_incompatible_syntax,
    rust_2021_prelude_collisions,
    single_use_lifetimes,
    trivial_casts,
    trivial_numeric_casts,
    unreachable_pub,
    unsafe_code,
    unsafe_op_in_unsafe_fn,
    unused_crate_dependencies,
    unused_extern_crates,
    unused_import_braces,
    unused_lifetimes,
    unused_qualifications,
    unused_results,
    warnings
)]

mod json;

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
use stac::{FlatItem, Item, ItemCollection};
use std::sync::Arc;
use thiserror::Error;

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

/// Crate specific error enum.
#[derive(Debug, Error)]
pub enum Error {
    /// [arrow_schema::ArrowError]
    #[error(transparent)]
    Arrow(#[from] arrow_schema::ArrowError),

    /// [geoarrow::error::GeoArrowError]
    #[error(transparent)]
    GeoArrow(#[from] geoarrow::error::GeoArrowError),

    /// Invalid bbox length.
    #[error("invalid bbox length (should be four or six): {0}")]
    InvalidBBoxLength(usize),

    /// No geometry column.
    #[error("no geometry column")]
    NoGeometryColumn,

    /// No items to serialize.
    #[error("no items")]
    NoItems,

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// XYZ dimensions are not supported.
    #[error("xyz dimensions not supported")]
    XYZNotSupported,
}

/// Crate-specific result type.
pub type Result<T> = std::result::Result<T, Error>;

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
/// let item = stac::read("data/simple-item.json").unwrap();
/// let item_collection = ItemCollection::from(vec![item]);
/// let table = stac_arrow::to_table(item_collection).unwrap();
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
                    return Err(Error::InvalidBBoxLength(bbox.len()));
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
/// use std::fs::File;
/// use geoarrow::io::parquet::GeoParquetRecordBatchReaderBuilder;
///
/// let file = File::open("examples/extended-item.parquet").unwrap();
/// let reader = GeoParquetRecordBatchReaderBuilder::try_new(file)
///     .unwrap()
///     .build()
///     .unwrap();
/// let table = reader.read_table().unwrap();
/// let item_collection = stac_arrow::from_table(table).unwrap();
/// ```
pub fn from_table(table: Table) -> Result<ItemCollection> {
    use GeoDataType::*;

    let (index, _) = table
        .schema()
        .column_with_name("geometry")
        .ok_or(Error::NoGeometryColumn)?;
    let mut json_rows = json::record_batches_to_json_rows(table.batches(), index)?;
    let mut items = Vec::new();
    for chunk in table.geometry_column(Some(index))?.geometry_chunks() {
        for i in 0..chunk.len() {
            let value = match chunk.data_type() {
                Point(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_point_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                LineString(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_line_string_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                LargeLineString(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_large_line_string_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                Polygon(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_polygon_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                LargePolygon(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_large_polygon_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                MultiPoint(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_multi_point_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                LargeMultiPoint(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_large_multi_point_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                MultiLineString(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_multi_line_string_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                LargeMultiLineString(_, dimension) => match dimension {
                    Dimension::XY => {
                        Value::from(&chunk.as_large_multi_line_string_2d().value_as_geo(i))
                    }
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                MultiPolygon(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_multi_polygon_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                LargeMultiPolygon(_, dimension) => match dimension {
                    Dimension::XY => {
                        Value::from(&chunk.as_large_multi_polygon_2d().value_as_geo(i))
                    }
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                Mixed(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_mixed_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                LargeMixed(_, dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_large_mixed_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                GeometryCollection(_, dimension) => match dimension {
                    Dimension::XY => {
                        Value::from(&chunk.as_geometry_collection_2d().value_as_geo(i))
                    }
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                LargeGeometryCollection(_, dimension) => match dimension {
                    Dimension::XY => {
                        Value::from(&chunk.as_large_geometry_collection_2d().value_as_geo(i))
                    }
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
                },
                WKB => Value::from(&chunk.as_wkb().value_as_geo(i)),
                LargeWKB => Value::from(&chunk.as_large_wkb().value_as_geo(i)),
                Rect(dimension) => match dimension {
                    Dimension::XY => Value::from(&chunk.as_rect_2d().value_as_geo(i)),
                    Dimension::XYZ => return Err(Error::XYZNotSupported),
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

#[cfg(test)]
mod tests {
    use geoarrow::io::parquet::GeoParquetRecordBatchReaderBuilder;
    use stac_validate::Validate;
    use std::fs::File;

    #[test]
    fn to_table() {
        let item = stac::read("data/simple-item.json").unwrap();
        let _ = super::to_table(vec![item].into()).unwrap();
    }

    #[test]
    fn from_table() {
        let file = File::open("examples/extended-item.parquet").unwrap();
        let reader = GeoParquetRecordBatchReaderBuilder::try_new(file)
            .unwrap()
            .build()
            .unwrap();
        let table = reader.read_table().unwrap();
        let item_collection = super::from_table(table).unwrap();
        assert_eq!(item_collection.items.len(), 1);
        item_collection.items[0].validate().unwrap();
    }

    #[test]
    fn roundtrip() {
        let item = stac::read("data/simple-item.json").unwrap();
        let table = super::to_table(vec![item].into()).unwrap();
        let _ = super::from_table(table).unwrap();
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
