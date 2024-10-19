//! The Projection extension.

use super::Extension;
use geojson::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The projection extension fields.
#[derive(Debug, Serialize, Deserialize, Default, PartialEq, Clone)]
pub struct Projection {
    /// EPSG code of the datasource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,

    /// WKT2 string representing the Coordinate Reference System (CRS) that the
    /// proj:geometry and proj:bbox fields represent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub wkt2: Option<String>,

    /// PROJJSON object representing the Coordinate Reference System (CRS) that
    /// the proj:geometry and proj:bbox fields represent
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projjson: Option<Map<String, Value>>,

    /// Defines the footprint of this Item.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub geometry: Option<Geometry>,

    /// Bounding box of the Item in the asset CRS in 2 or 3 dimensions.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bbox: Option<Vec<f64>>,

    /// Coordinates representing the centroid of the Item (in lat/long)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub centroid: Option<Centroid>,

    /// Number of pixels in Y and X directions for the default grid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shape: Option<Vec<usize>>,

    /// The affine transformation coefficients for the default grid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Vec<f64>>,
}

/// This object represents the centroid of the Item Geometry.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Centroid {
    /// The latitude of the centroid.
    pub lat: f64,

    /// The longitude of the centroid.
    pub lon: f64,
}

impl Projection {
    /// Returns true if this projection structure is empty.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_extensions::Projection;
    ///
    /// let projection = Projection::default();
    /// assert!(projection.is_empty());
    /// ```
    pub fn is_empty(&self) -> bool {
        serde_json::to_value(self)
            .map(|v| v == Value::Object(Default::default()))
            .unwrap_or(true)
    }
}

impl Extension for Projection {
    const IDENTIFIER: &'static str =
        "https://stac-extensions.github.io/projection/v2.0.0/schema.json";
    const PREFIX: &'static str = "proj";
}

#[cfg(test)]
mod tests {
    use super::Projection;
    use crate::{Extensions, Item};

    #[test]
    fn example() {
        let item: Item =
            stac::read("examples/extensions-collection/proj-example/proj-example.json").unwrap();
        let projection = item.extension::<Projection>().unwrap();
        assert_eq!(projection.code.unwrap(), "EPSG:32614");
    }
}
