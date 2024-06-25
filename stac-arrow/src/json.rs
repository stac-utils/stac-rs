//! Functions taken from [arrow-json
//! v51](https://github.com/apache/arrow-rs/blob/51.0.0/arrow-json/src/writer.rs).
//!
//! In particular, the `record_batches_to_json_rows` was deprecated and
//! subsequently removed in v52, but it turns out that's the functionality we
//! need to go to [Items](stac::Item). We've modified the functions to handle
//! geometries as well.

use arrow::{
    array::{
        as_boolean_array, as_fixed_size_list_array, as_large_list_array, as_largestring_array,
        as_list_array, as_map_array, as_string_array, Array, ArrayRef, AsArray, StructArray,
    },
    datatypes::{
        ArrowPrimitiveType, DataType, Float16Type, Float32Type, Float64Type, Int16Type, Int32Type,
        Int64Type, Int8Type, UInt16Type, UInt32Type, UInt64Type, UInt8Type,
    },
    error::ArrowError,
    json::JsonSerializable,
    util::display::{ArrayFormatter, FormatOptions},
};
use geoarrow::{
    array::AsGeometryArray, datatypes::GeoDataType, schema::GeoSchemaExt, table::Table,
    trait_::GeometryArrayAccessor, GeometryArrayTrait,
};
use geojson::Geometry;
use serde_json::{Map, Value};
use std::sync::Arc;

macro_rules! set_column_by_array_type {
    ($cast_fn:ident, $col_name:ident, $rows:ident, $array:ident, $explicit_nulls:ident) => {
        let arr = $cast_fn($array);
        $rows
            .iter_mut()
            .zip(arr.iter())
            .filter_map(|(maybe_row, maybe_value)| maybe_row.as_mut().map(|row| (row, maybe_value)))
            .for_each(|(row, maybe_value)| {
                if let Some(j) = maybe_value.map(Into::into) {
                    row.insert($col_name.to_string(), j);
                } else if $explicit_nulls {
                    row.insert($col_name.to_string(), Value::Null);
                }
            });
    };
}

fn set_column_for_json_rows(
    rows: &mut [Option<Map<String, Value>>],
    array: &ArrayRef,
    col_name: &str,
    explicit_nulls: bool,
) -> Result<(), ArrowError> {
    match array.data_type() {
        DataType::Int8 => {
            set_column_by_primitive_type::<Int8Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::Int16 => {
            set_column_by_primitive_type::<Int16Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::Int32 => {
            set_column_by_primitive_type::<Int32Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::Int64 => {
            set_column_by_primitive_type::<Int64Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::UInt8 => {
            set_column_by_primitive_type::<UInt8Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::UInt16 => {
            set_column_by_primitive_type::<UInt16Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::UInt32 => {
            set_column_by_primitive_type::<UInt32Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::UInt64 => {
            set_column_by_primitive_type::<UInt64Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::Float16 => {
            set_column_by_primitive_type::<Float16Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::Float32 => {
            set_column_by_primitive_type::<Float32Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::Float64 => {
            set_column_by_primitive_type::<Float64Type>(rows, array, col_name, explicit_nulls);
        }
        DataType::Null => {
            if explicit_nulls {
                rows.iter_mut()
                    .filter_map(|maybe_row| maybe_row.as_mut())
                    .for_each(|row| {
                        row.insert(col_name.to_string(), Value::Null);
                    });
            }
        }
        DataType::Boolean => {
            set_column_by_array_type!(as_boolean_array, col_name, rows, array, explicit_nulls);
        }
        DataType::Utf8 => {
            set_column_by_array_type!(as_string_array, col_name, rows, array, explicit_nulls);
        }
        DataType::LargeUtf8 => {
            set_column_by_array_type!(as_largestring_array, col_name, rows, array, explicit_nulls);
        }
        DataType::Date32
        | DataType::Date64
        | DataType::Timestamp(_, _)
        | DataType::Time32(_)
        | DataType::Time64(_)
        | DataType::Duration(_) => {
            let options = FormatOptions::default();
            let formatter = ArrayFormatter::try_new(array.as_ref(), &options)?;
            let nulls = array.nulls();
            rows.iter_mut()
                .enumerate()
                .filter_map(|(idx, maybe_row)| maybe_row.as_mut().map(|row| (idx, row)))
                .for_each(|(idx, row)| {
                    let maybe_value = nulls
                        .map(|x| x.is_valid(idx))
                        .unwrap_or(true)
                        .then(|| formatter.value(idx).to_string().into());
                    if let Some(j) = maybe_value {
                        row.insert(col_name.to_string(), j);
                    } else if explicit_nulls {
                        row.insert(col_name.to_string(), Value::Null);
                    }
                });
        }
        DataType::Struct(_) => {
            let inner_objs = struct_array_to_jsonmap_array(array.as_struct(), explicit_nulls)?;
            rows.iter_mut()
                .zip(inner_objs)
                .filter_map(|(maybe_row, maybe_obj)| maybe_row.as_mut().map(|row| (row, maybe_obj)))
                .for_each(|(row, maybe_obj)| {
                    let json = if let Some(obj) = maybe_obj {
                        Value::Object(obj)
                    } else {
                        Value::Null
                    };
                    row.insert(col_name.to_string(), json);
                });
        }
        DataType::List(_) => {
            let listarr = as_list_array(array);
            rows.iter_mut()
                .zip(listarr.iter())
                .filter_map(|(maybe_row, maybe_value)| {
                    maybe_row.as_mut().map(|row| (row, maybe_value))
                })
                .try_for_each(|(row, maybe_value)| -> Result<(), ArrowError> {
                    let maybe_value = maybe_value
                        .map(|v| array_to_json_array_internal(&v, explicit_nulls).map(Value::Array))
                        .transpose()?;
                    if let Some(j) = maybe_value {
                        row.insert(col_name.to_string(), j);
                    } else if explicit_nulls {
                        row.insert(col_name.to_string(), Value::Null);
                    }
                    Ok(())
                })?;
        }
        DataType::LargeList(_) => {
            let listarr = as_large_list_array(array);
            rows.iter_mut()
                .zip(listarr.iter())
                .filter_map(|(maybe_row, maybe_value)| {
                    maybe_row.as_mut().map(|row| (row, maybe_value))
                })
                .try_for_each(|(row, maybe_value)| -> Result<(), ArrowError> {
                    let maybe_value = maybe_value
                        .map(|v| array_to_json_array_internal(&v, explicit_nulls).map(Value::Array))
                        .transpose()?;
                    if let Some(j) = maybe_value {
                        row.insert(col_name.to_string(), j);
                    } else if explicit_nulls {
                        row.insert(col_name.to_string(), Value::Null);
                    }
                    Ok(())
                })?;
        }
        DataType::Dictionary(_, value_type) => {
            let hydrated = arrow_cast::cast::cast(&array, value_type)
                .expect("cannot cast dictionary to underlying values");
            set_column_for_json_rows(rows, &hydrated, col_name, explicit_nulls)?;
        }
        DataType::Map(_, _) => {
            let maparr = as_map_array(array);

            let keys = maparr.keys();
            let values = maparr.values();

            // Keys have to be strings to convert to json.
            if !matches!(keys.data_type(), DataType::Utf8) {
                return Err(ArrowError::JsonError(format!(
                    "data type {:?} not supported in nested map for json writer",
                    keys.data_type()
                )));
            }

            let keys = keys.as_string::<i32>();
            let values = array_to_json_array_internal(values, explicit_nulls)?;

            let mut kv = keys.iter().zip(values);

            for (i, row) in rows
                .iter_mut()
                .enumerate()
                .filter_map(|(i, maybe_row)| maybe_row.as_mut().map(|row| (i, row)))
            {
                if maparr.is_null(i) {
                    row.insert(col_name.to_string(), serde_json::Value::Null);
                    continue;
                }

                let len = maparr.value_length(i) as usize;
                let mut obj = serde_json::Map::new();

                for (_, (k, v)) in (0..len).zip(&mut kv) {
                    obj.insert(k.expect("keys in a map should be non-null").to_string(), v);
                }

                row.insert(col_name.to_string(), serde_json::Value::Object(obj));
            }
        }
        _ => {
            return Err(ArrowError::JsonError(format!(
                "data type {:?} not supported in nested map for json writer",
                array.data_type()
            )))
        }
    }
    Ok(())
}

fn set_column_by_primitive_type<T>(
    rows: &mut [Option<Map<String, Value>>],
    array: &ArrayRef,
    col_name: &str,
    explicit_nulls: bool,
) where
    T: ArrowPrimitiveType,
    T::Native: JsonSerializable,
{
    let primitive_arr = array.as_primitive::<T>();

    rows.iter_mut()
        .zip(primitive_arr.iter())
        .filter_map(|(maybe_row, maybe_value)| maybe_row.as_mut().map(|row| (row, maybe_value)))
        .for_each(|(row, maybe_value)| {
            if let Some(j) = maybe_value.and_then(|v| v.into_json_value()) {
                row.insert(col_name.to_string(), j);
            } else if explicit_nulls {
                row.insert(col_name.to_string(), Value::Null);
            }
        });
}

fn struct_array_to_jsonmap_array(
    array: &StructArray,
    explicit_nulls: bool,
) -> Result<Vec<Option<Map<String, Value>>>, ArrowError> {
    let inner_col_names = array.column_names();

    let mut inner_objs = (0..array.len())
        // Ensure we write nulls for struct arrays as nulls in JSON
        // Instead of writing a struct with nulls
        .map(|index| array.is_valid(index).then(Map::new))
        .collect::<Vec<Option<Map<String, Value>>>>();

    for (j, struct_col) in array.columns().iter().enumerate() {
        set_column_for_json_rows(
            &mut inner_objs,
            struct_col,
            inner_col_names[j],
            explicit_nulls,
        )?
    }
    Ok(inner_objs)
}

fn array_to_json_array_internal(
    array: &dyn Array,
    explicit_nulls: bool,
) -> Result<Vec<Value>, ArrowError> {
    match array.data_type() {
        DataType::Null => Ok(std::iter::repeat(Value::Null).take(array.len()).collect()),
        DataType::Boolean => Ok(array
            .as_boolean()
            .iter()
            .map(|maybe_value| match maybe_value {
                Some(v) => v.into(),
                None => Value::Null,
            })
            .collect()),

        DataType::Utf8 => Ok(array
            .as_string::<i32>()
            .iter()
            .map(|maybe_value| match maybe_value {
                Some(v) => v.into(),
                None => Value::Null,
            })
            .collect()),
        DataType::LargeUtf8 => Ok(array
            .as_string::<i64>()
            .iter()
            .map(|maybe_value| match maybe_value {
                Some(v) => v.into(),
                None => Value::Null,
            })
            .collect()),
        DataType::Int8 => primitive_array_to_json::<Int8Type>(array),
        DataType::Int16 => primitive_array_to_json::<Int16Type>(array),
        DataType::Int32 => primitive_array_to_json::<Int32Type>(array),
        DataType::Int64 => primitive_array_to_json::<Int64Type>(array),
        DataType::UInt8 => primitive_array_to_json::<UInt8Type>(array),
        DataType::UInt16 => primitive_array_to_json::<UInt16Type>(array),
        DataType::UInt32 => primitive_array_to_json::<UInt32Type>(array),
        DataType::UInt64 => primitive_array_to_json::<UInt64Type>(array),
        DataType::Float16 => primitive_array_to_json::<Float16Type>(array),
        DataType::Float32 => primitive_array_to_json::<Float32Type>(array),
        DataType::Float64 => primitive_array_to_json::<Float64Type>(array),
        DataType::List(_) => as_list_array(array)
            .iter()
            .map(|maybe_value| match maybe_value {
                Some(v) => Ok(Value::Array(array_to_json_array_internal(
                    &v,
                    explicit_nulls,
                )?)),
                None => Ok(Value::Null),
            })
            .collect(),
        DataType::LargeList(_) => as_large_list_array(array)
            .iter()
            .map(|maybe_value| match maybe_value {
                Some(v) => Ok(Value::Array(array_to_json_array_internal(
                    &v,
                    explicit_nulls,
                )?)),
                None => Ok(Value::Null),
            })
            .collect(),
        DataType::FixedSizeList(_, _) => as_fixed_size_list_array(array)
            .iter()
            .map(|maybe_value| match maybe_value {
                Some(v) => Ok(Value::Array(array_to_json_array_internal(
                    &v,
                    explicit_nulls,
                )?)),
                None => Ok(Value::Null),
            })
            .collect(),
        DataType::Struct(_) => {
            let jsonmaps = struct_array_to_jsonmap_array(array.as_struct(), explicit_nulls)?;
            let json_values = jsonmaps
                .into_iter()
                .map(|maybe_map| maybe_map.map(Value::Object).unwrap_or(Value::Null))
                .collect();
            Ok(json_values)
        }
        DataType::Map(_, _) => as_map_array(array)
            .iter()
            .map(|maybe_value| match maybe_value {
                Some(v) => Ok(Value::Array(array_to_json_array_internal(
                    &v,
                    explicit_nulls,
                )?)),
                None => Ok(Value::Null),
            })
            .collect(),
        t => Err(ArrowError::JsonError(format!(
            "data type {t:?} not supported"
        ))),
    }
}

fn primitive_array_to_json<T>(array: &dyn Array) -> Result<Vec<Value>, ArrowError>
where
    T: ArrowPrimitiveType,
    T::Native: JsonSerializable,
{
    Ok(array
        .as_primitive::<T>()
        .iter()
        .map(|maybe_value| match maybe_value {
            Some(v) => v.into_json_value().unwrap_or(Value::Null),
            None => Value::Null,
        })
        .collect())
}

fn set_geometry_column_for_json_rows(
    rows: &mut [Option<Map<String, Value>>],
    array: Arc<dyn GeometryArrayTrait>,
    col_name: &str,
) -> Result<(), ArrowError> {
    match array.data_type() {
        GeoDataType::Point(_) => set_column_by_geometry(rows, array.as_ref().as_point(), col_name),
        GeoDataType::LineString(_) => {
            set_column_by_geometry(rows, array.as_ref().as_line_string(), col_name)
        }
        GeoDataType::LargeLineString(_) => {
            set_column_by_geometry(rows, array.as_ref().as_large_line_string(), col_name)
        }
        GeoDataType::Polygon(_) => {
            set_column_by_geometry(rows, array.as_ref().as_polygon(), col_name)
        }
        GeoDataType::LargePolygon(_) => {
            set_column_by_geometry(rows, array.as_ref().as_large_polygon(), col_name)
        }
        GeoDataType::MultiPoint(_) => {
            set_column_by_geometry(rows, array.as_ref().as_multi_point(), col_name)
        }
        GeoDataType::LargeMultiPoint(_) => {
            set_column_by_geometry(rows, array.as_ref().as_large_multi_point(), col_name)
        }
        GeoDataType::MultiLineString(_) => {
            set_column_by_geometry(rows, array.as_ref().as_multi_line_string(), col_name)
        }
        GeoDataType::LargeMultiLineString(_) => {
            set_column_by_geometry(rows, array.as_ref().as_large_multi_line_string(), col_name)
        }
        GeoDataType::MultiPolygon(_) => {
            set_column_by_geometry(rows, array.as_ref().as_multi_polygon(), col_name)
        }
        GeoDataType::LargeMultiPolygon(_) => {
            set_column_by_geometry(rows, array.as_ref().as_large_multi_polygon(), col_name)
        }
        GeoDataType::Mixed(_) => set_column_by_geometry(rows, array.as_ref().as_mixed(), col_name),
        GeoDataType::LargeMixed(_) => {
            set_column_by_geometry(rows, array.as_ref().as_large_mixed(), col_name)
        }
        GeoDataType::GeometryCollection(_) => {
            set_column_by_geometry(rows, array.as_ref().as_geometry_collection(), col_name)
        }
        GeoDataType::LargeGeometryCollection(_) => set_column_by_geometry(
            rows,
            array.as_ref().as_large_geometry_collection(),
            col_name,
        ),
        GeoDataType::Rect => set_column_by_geometry(rows, array.as_ref().as_rect(), col_name),
        GeoDataType::WKB => set_column_by_geometry(rows, array.as_ref().as_wkb(), col_name),
        GeoDataType::LargeWKB => {
            set_column_by_geometry(rows, array.as_ref().as_large_wkb(), col_name)
        }
    }
    Ok(())
}

fn set_column_by_geometry<'a>(
    rows: &mut [Option<Map<String, Value>>],
    array: &impl GeometryArrayAccessor<'a>,
    col_name: &str,
) {
    rows.iter_mut()
        .zip(array.as_ref().as_polygon().iter_geo())
        .filter_map(|(maybe_row, maybe_value)| {
            maybe_row.as_mut().and_then(|row| {
                maybe_value
                    .and_then(|value| serde_json::to_value(Geometry::from(&value)).ok())
                    .map(|value| (row, value))
            })
        })
        .for_each(|(row, value)| {
            row.insert(col_name.to_string(), value);
        })
}

pub(crate) fn table_to_json_rows(
    table: Table,
) -> Result<impl Iterator<Item = Map<String, Value>>, crate::Error> {
    let batches = table.batches();
    // TODO can we remove the option around the map?
    let mut rows: Vec<Option<Map<String, Value>>> = std::iter::repeat(Some(Map::new()))
        .take(batches.iter().map(|b| b.num_rows()).sum())
        .collect();

    if !rows.is_empty() {
        let schema = batches[0].schema();
        let geometry_column_indices = schema.as_ref().geometry_columns();
        let mut base = 0;
        for batch in batches {
            let row_count = batch.num_rows();
            let row_slice = &mut rows[base..base + batch.num_rows()];
            for (j, col) in batch.columns().iter().enumerate() {
                let col_name = schema.field(j).name();
                if geometry_column_indices.contains(&j) {
                    let col = geoarrow::array::from_arrow_array(col, schema.field(j))?;
                    crate::json::set_geometry_column_for_json_rows(row_slice, col, col_name)?
                } else {
                    crate::json::set_column_for_json_rows(row_slice, col, col_name, false)?
                }
            }
            base += row_count;
        }
    }
    Ok(rows.into_iter().map(|a| a.unwrap()))
}
