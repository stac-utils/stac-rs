use crate::{Error, Result};
use gdal::spatial_ref::{CoordTransform, SpatialRef};
use serde_json::Value;
use stac::Bbox;
use stac_extensions::Projection;

pub trait ProjectionCalculations {
    fn wgs84_bounds(&self) -> Result<Option<Bbox>>;
    fn spatial_ref(&self) -> Result<Option<SpatialRef>>;
}

/// Calculations based on GDAL for projection.
///
/// # Examples:
/// ```
/// use stac::Bbox;
/// use stac_extensions::Projection;
/// use stac_gdal::projection::ProjectionCalculations;
/// let mut projection = Projection::default();
/// projection.code = Some("EPSG:32633".to_owned());
/// projection.bbox = Some(Bbox::from([399960.0, 4090200.0, 509760.0, 4200000.0]).into());
/// assert_eq!(
///     projection.wgs84_bounds().unwrap(),
///     Some(Bbox::new(
///         36.9525649,
///         13.8614709,
///         37.9475895,
///         15.1110860
///     ))
/// );
/// ```
impl ProjectionCalculations for Projection {
    fn spatial_ref(&self) -> Result<Option<SpatialRef>> {
        if self.code.as_ref().is_some_and(|c| c.starts_with("EPSG:")) {
            let code = self
                .code
                .as_ref()
                .and_then(|c| c.strip_prefix("EPSG:"))
                .ok_or(Error::ParseEPSGProjectionError(
                    self.code.as_ref().unwrap().to_string(),
                ))?
                .parse()?;

            SpatialRef::from_epsg(code).map(Some).map_err(Error::from)
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

    fn wgs84_bounds(&self) -> Result<Option<Bbox>> {
        if let Some(bbox) = self.bbox.as_ref() {
            if bbox.len() != 4 {
                return Ok(None);
            }
            if let Some(spatial_ref) = self.spatial_ref()? {
                let wgs84 = SpatialRef::from_epsg(4326)?;
                let coord_transform = CoordTransform::new(&spatial_ref, &wgs84)?;
                let bounds =
                    coord_transform.transform_bounds(&[bbox[0], bbox[1], bbox[2], bbox[3]], 21)?;
                let [x1, y1, x2, y2] = bounds;
                let round = |n: f64| (n * 10_000_000.).round() / 10_000_000.;
                Ok(Some(Bbox::new(
                    round(x1.min(x2)),
                    round(y1.min(y2)),
                    round(x1.max(x2)),
                    round(y1.max(y2)),
                )))
            } else {
                Ok(None)
            }
        } else {
            Ok(None)
        }
    }
}
