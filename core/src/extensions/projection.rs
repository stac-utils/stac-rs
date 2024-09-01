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
    /// Returns this projection's bounds in WGS84.
    ///
    /// Requires one of the crs fields to be set (epsg, wkt2, or projjson) as well as a bbox.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::extensions::Projection;
    /// let projection = Projection {
    ///     epsg: Some(32621),
    ///     bbox: Some(vec![
    ///         373185.0,
    ///         8019284.949381611,
    ///         639014.9492102272,
    ///         8286015.0
    ///     ]),
    ///     ..Default::default()
    /// };
    /// let bounds = projection.wgs84_bounds().unwrap().unwrap();
    /// ```
    #[cfg(feature = "gdal")]
    pub fn wgs84_bounds(&self) -> crate::Result<Option<crate::Bbox>> {
        use gdal::spatial_ref::{AxisMappingStrategy, CoordTransform, SpatialRef};

        if let Some(bbox) = self.bbox.as_ref() {
            if bbox.len() != 4 {
                return Ok(None);
            }
            if let Some(spatial_ref) = self.spatial_ref()? {
                let mut wgs84 = SpatialRef::from_epsg(4326)?;
                // Ensure we're lon then lat
                wgs84.set_axis_mapping_strategy(AxisMappingStrategy::TraditionalGisOrder);
                let coord_transform = CoordTransform::new(&spatial_ref, &wgs84)?;
                let bounds =
                    coord_transform.transform_bounds(&[bbox[0], bbox[1], bbox[2], bbox[3]], 21)?;
                let round = |n: f64| (n * 10_000_000.).round() / 10_000_000.;
                Ok(Some(crate::Bbox::new(
                    round(bounds[0]),
                    round(bounds[1]),
                    round(bounds[2]),
                    round(bounds[3]),
                )))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }

    #[cfg(feature = "gdal")]
    fn spatial_ref(&self) -> crate::Result<Option<gdal::spatial_ref::SpatialRef>> {
        use crate::Error;
        use gdal::spatial_ref::SpatialRef;

        if let Some(epsg) = self.epsg {
            SpatialRef::from_epsg(epsg.try_into()?)
                .map(Some)
                .map_err(Error::from)
        } else if let Some(wkt) = self.wkt2.as_ref() {
            SpatialRef::from_wkt(wkt).map(Some).map_err(Error::from)
        } else if let Some(projjson) = self.projjson.clone() {
            SpatialRef::from_definition(&serde_json::to_string(&Value::Object(projjson))?)
                .map(Some)
                .map_err(Error::from)
        } else {
            Ok(None)
        }
    }
}

impl Extension for Projection {
    const IDENTIFIER: &'static str =
        "https://stac-extensions.github.io/projection/v1.1.0/schema.json";
    const PREFIX: &'static str = "proj";
}

#[cfg(test)]
mod tests {
    #[cfg(feature = "gdal")]
    #[test]
    fn axis_order() {
        use super::Projection;

        let projection = Projection {
            epsg: Some(32621),
            bbox: Some(vec![
                373185.0,
                8019284.949381611,
                639014.9492102272,
                8286015.0,
            ]),
            ..Default::default()
        };
        let bounds = projection.wgs84_bounds().unwrap().unwrap();
        assert!(
            (bounds.xmin() - -61.2876244).abs() < 0.1,
            "{}",
            bounds.xmin()
        );
        assert!((bounds.ymin() - 72.229798).abs() < 0.1);
    }
}
