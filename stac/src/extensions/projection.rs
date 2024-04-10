//! The Projection extension.

use crate::Geometry;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

use super::Extension;

/// The projection extension fields.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Projection {
    /// EPSG code of the datasource
    #[serde(skip_serializing_if = "Option::is_none")]
    pub epsg: Option<i32>,

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
    pub shape: Option<Vec<f64>>,

    /// The affine transformation coefficients for the default grid
    #[serde(skip_serializing_if = "Option::is_none")]
    pub transform: Option<Vec<f64>>,
}

/// This object represents the centroid of the Item Geometry.
#[derive(Debug, Serialize, Deserialize)]
pub struct Centroid {
    /// The latitude of the centroid.
    pub lat: f64,

    /// The longitude of the centroid.
    pub lon: f64,
}

impl Extension for Projection {
    const IDENTIFIER: &'static str =
        "https://stac-extensions.github.io/projection/v1.1.0/schema.json";
    const PREFIX: &'static str = "proj";
}
