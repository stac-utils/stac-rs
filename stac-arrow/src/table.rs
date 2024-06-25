use crate::{Error, Result};
use geoarrow::table::Table;
use serde_json::{Map, Value};
use stac::{FlatItem, Item};

/// Converts a [geoarrow::table::Table] to a vector of [Items](stac::Item).
///
/// # Examples
///
/// ```
/// # #[cfg(feature = "parquet-compression")] // Needed to read the parquet file
/// # {
/// let file = std::fs::File::open("examples/sentinel-2-l2a-1.1.0.parquet").unwrap();
/// let table = geoarrow::io::parquet::read_geoparquet(file, Default::default()).unwrap();
/// let items = stac_arrow::table_to_items(table).unwrap();
/// assert_eq!(items.len(), 10);
/// # }
/// ```
pub fn table_to_items(table: Table) -> Result<Vec<Item>> {
    crate::json::table_to_json_rows(table)?
        .map(|mut row| -> Result<_> {
            if let Some(bbox) = bbox_to_vec(&row)? {
                row.insert("bbox".to_string(), bbox.into());
            }
            let flat_item: FlatItem = serde_json::from_value(Value::Object(row))?;
            Item::try_from(flat_item).map_err(Error::from)
        })
        .collect()
}

fn bbox_to_vec(row: &Map<String, Value>) -> Result<Option<Vec<f64>>> {
    if let Some(bbox) = row.get("bbox") {
        if let Value::Object(bbox) = bbox {
            let xmin = bbox
                .get("xmin")
                .and_then(|xmin| xmin.as_f64())
                .ok_or_else(|| Error::InvalidBBoxMap(bbox.clone()))?;
            let xmax = bbox
                .get("xmax")
                .and_then(|xmax| xmax.as_f64())
                .ok_or_else(|| Error::InvalidBBoxMap(bbox.clone()))?;
            let ymin = bbox
                .get("ymin")
                .and_then(|ymin| ymin.as_f64())
                .ok_or_else(|| Error::InvalidBBoxMap(bbox.clone()))?;
            let ymax = bbox
                .get("ymax")
                .and_then(|ymax| ymax.as_f64())
                .ok_or_else(|| Error::InvalidBBoxMap(bbox.clone()))?;
            if let Some((zmin, zmax)) =
                bbox.get("zmin")
                    .and_then(|zmin| zmin.as_f64())
                    .and_then(|zmin| {
                        bbox.get("zmax")
                            .and_then(|zmax| zmax.as_f64())
                            .map(|zmax| (zmin, zmax))
                    })
            {
                Ok(Some(vec![xmin, ymin, zmin, xmax, ymax, zmax]))
            } else {
                Ok(Some(vec![xmin, ymin, xmax, ymax]))
            }
        } else {
            Err(Error::BBoxIsNotAMap(bbox.clone()))
        }
    } else {
        Ok(None)
    }
}
