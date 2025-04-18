// Licensed to the Apache Software Foundation (ASF) under one
// or more contributor license agreements.  See the NOTICE file
// distributed with this work for additional information
// regarding copyright ownership.  The ASF licenses this file
// to you under the Apache License, Version 2.0 (the
// "License"); you may not use this file except in compliance
// with the License.  You may obtain a copy of the License at
//
//   http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing,
// software distributed under the License is distributed on an
// "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY
// KIND, either express or implied.  See the License for the
// specific language governing permissions and limitations
// under the License.

//! Utilities for working with JSON and (geo)arrow.
//!
//! Taken from v51.0.0 of
//! [arrow-json](https://docs.rs/arrow-json/51.0.0/arrow_json/index.html), we've
//! lifted this code to convert record batches to vectors of
//! [serde_json::Value]. We've been able to go _mostly_ as-is, but there's some
//! modifications and cutouts.

#![allow(unused_results)]

const TOP_LEVEL_KEYS: [&str; 10] = [
    "type",
    "stac_version",
    "stac_extensions",
    "id",
    "geometry",
    "bbox",
    "properties",
    "links",
    "assets",
    "collection",
];

use arrow_array::{cast::*, types::*, *};
use arrow_cast::display::{ArrayFormatter, FormatOptions};
use arrow_json::JsonSerializable;
use arrow_schema::*;
use geoarrow::table::Table;
use serde_json::{Value, json, map::Map as JsonMap};
use std::iter;

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

fn struct_array_to_jsonmap_array(
    array: &StructArray,
    explicit_nulls: bool,
) -> Result<Vec<Option<JsonMap<String, Value>>>, ArrowError> {
    let inner_col_names = array.column_names();

    let mut inner_objs = (0..array.len())
        // Ensure we write nulls for struct arrays as nulls in JSON
        // Instead of writing a struct with nulls
        .map(|index| array.is_valid(index).then(JsonMap::new))
        .collect::<Vec<Option<JsonMap<String, Value>>>>();

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
        DataType::Null => Ok(iter::repeat_n(Value::Null, array.len()).collect()),
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

fn set_column_by_primitive_type<T>(
    rows: &mut [Option<JsonMap<String, Value>>],
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
        .for_each(
            |(row, maybe_value)| match maybe_value.and_then(|v| v.into_json_value()) {
                Some(j) => {
                    row.insert(col_name.to_string(), j);
                }
                _ => {
                    if explicit_nulls {
                        row.insert(col_name.to_string(), Value::Null);
                    }
                }
            },
        );
}

fn set_column_for_json_rows(
    rows: &mut [Option<JsonMap<String, Value>>],
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
                        if col_name == "bbox" {
                            convert_bbox(obj)
                        } else {
                            Value::Object(obj)
                        }
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
                    row.insert(col_name.to_string(), Value::Null);
                    continue;
                }

                let len = maparr.value_length(i) as usize;
                let mut obj = serde_json::Map::new();

                for (_, (k, v)) in (0..len).zip(&mut kv) {
                    obj.insert(k.expect("keys in a map should be non-null").to_string(), v);
                }

                row.insert(col_name.to_string(), Value::Object(obj));
            }
        }
        _ => {
            return Err(ArrowError::JsonError(format!(
                "data type {:?} not supported in nested map for json writer",
                array.data_type()
            )));
        }
    }
    Ok(())
}

/// Converts a table to json rows.
pub fn from_table(table: Table) -> Result<Vec<serde_json::Map<String, Value>>, crate::Error> {
    use geoarrow::{array::AsNativeArray, datatypes::NativeType::*, trait_::ArrayAccessor};
    use geojson::Value;

    let index = table
        .schema()
        .column_with_name("geometry")
        .map(|(index, _)| index);
    let mut json_rows = record_batches_to_json_rows(table.batches(), index)?;
    let mut items = Vec::new();
    if let Some(index) = index {
        for chunk in table.geometry_column(Some(index))?.geometry_chunks() {
            for i in 0..chunk.len() {
                let value = match chunk.data_type() {
                    Point(_) => Value::from(&chunk.as_ref().as_point().value_as_geo(i)),
                    LineString(_) => Value::from(&chunk.as_ref().as_line_string().value_as_geo(i)),
                    Polygon(_) => Value::from(&chunk.as_ref().as_polygon().value_as_geo(i)),
                    MultiPoint(_) => Value::from(&chunk.as_ref().as_multi_point().value_as_geo(i)),
                    MultiLineString(_) => {
                        Value::from(&chunk.as_ref().as_multi_line_string().value_as_geo(i))
                    }
                    MultiPolygon(_) => {
                        Value::from(&chunk.as_ref().as_multi_polygon().value_as_geo(i))
                    }
                    Geometry(_) => Value::from(&chunk.as_ref().as_geometry().value_as_geo(i)),
                    GeometryCollection(_) => {
                        Value::from(&chunk.as_ref().as_geometry_collection().value_as_geo(i))
                    }
                    Rect(_) => Value::from(&chunk.as_ref().as_rect().value_as_geo(i)),
                };
                let mut row = json_rows
                    .next()
                    .expect("we shouldn't run out of rows before we're done");
                let _ = row.insert(
                    "geometry".into(),
                    serde_json::to_value(geojson::Geometry::new(value))?,
                );
                items.push(unflatten(row));
            }
        }
    } else {
        items = json_rows.map(unflatten).collect();
    }
    Ok(items)
}

fn unflatten(mut item: serde_json::Map<String, Value>) -> serde_json::Map<String, Value> {
    let mut properties = serde_json::Map::new();
    let keys: Vec<_> = item
        .keys()
        .filter_map(|key| {
            if TOP_LEVEL_KEYS.contains(&key.as_str()) {
                None
            } else {
                Some(key.to_string())
            }
        })
        .collect();
    for key in keys {
        if let Some(value) = item.remove(&key) {
            let _ = properties.insert(key, value);
        }
    }
    if !properties.is_empty() {
        let _ = item.insert("properties".to_string(), Value::Object(properties));
    }
    item
}

fn record_batches_to_json_rows(
    batches: &[RecordBatch],
    geometry_index: Option<usize>,
) -> Result<impl Iterator<Item = JsonMap<String, Value>> + use<>, ArrowError> {
    // For backwards compatibility, default to skip nulls
    // Skip converting the geometry index, we'll do that later.
    record_batches_to_json_rows_internal(batches, false, geometry_index)
}

fn record_batches_to_json_rows_internal(
    batches: &[RecordBatch],
    explicit_nulls: bool,
    geometry_index: Option<usize>,
) -> Result<impl Iterator<Item = JsonMap<String, Value>> + use<>, ArrowError> {
    let mut rows: Vec<Option<JsonMap<String, Value>>> = iter::repeat_n(
        Some(JsonMap::new()),
        batches.iter().map(|b| b.num_rows()).sum(),
    )
    .collect();

    if !rows.is_empty() {
        let schema = batches[0].schema();
        let mut base = 0;
        for batch in batches {
            let row_count = batch.num_rows();
            let row_slice = &mut rows[base..base + batch.num_rows()];
            for (j, col) in batch.columns().iter().enumerate() {
                if geometry_index.map(|v| v == j).unwrap_or_default() {
                    continue;
                }
                let col_name = schema.field(j).name();
                set_column_for_json_rows(row_slice, col, col_name, explicit_nulls)?
            }
            base += row_count;
        }
    }

    Ok(rows.into_iter().map(|a| {
        let mut a = a.unwrap();
        if let Some(assets) = a.get_mut("assets").and_then(|a| a.as_object_mut()) {
            assets.retain(|_, asset| asset.is_object());
        }
        a
    }))
}

fn convert_bbox(obj: serde_json::Map<String, Value>) -> Value {
    if let Some((((xmin, ymin), xmax), ymax)) = obj
        .get("xmin")
        .and_then(|v| v.as_f64())
        .zip(obj.get("ymin").and_then(|v| v.as_f64()))
        .zip(obj.get("xmax").and_then(|v| v.as_f64()))
        .zip(obj.get("ymax").and_then(|v| v.as_f64()))
    {
        if let Some((zmin, zmax)) = obj
            .get("zmin")
            .and_then(|v| v.as_f64())
            .zip(obj.get("zmax").and_then(|v| v.as_f64()))
        {
            json!([xmin, ymin, zmin, xmax, ymax, zmax])
        } else {
            json!([xmin, ymin, xmax, ymax])
        }
    } else {
        Value::Object(obj)
    }
}
