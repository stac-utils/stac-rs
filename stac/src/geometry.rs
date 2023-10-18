use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

#[cfg(feature = "geo")]
use crate::{Error, Result};

/// Additional metadata fields can be added to the GeoJSON Object Properties.
///
/// We can't just use the [geojson] crate because it doesn't implement [schemars].
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
#[cfg_attr(feature = "schemars", derive(schemars::JsonSchema))]
pub struct Geometry {
    /// The geometry type.
    pub r#type: String,

    /// The other geometry attributes.
    ///
    /// `GeometryCollection` doesn't have a `coordinates` member, so we must
    /// capture everything in a flat, generic array.
    #[serde(flatten)]
    pub attributes: Map<String, Value>,
}

impl Geometry {
    /// Creates a point geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Geometry;
    /// let geometry = Geometry::point(-108.0, 42.0);
    /// ```
    pub fn point(x: f64, y: f64) -> Geometry {
        use serde_json::json;

        let mut attributes = Map::new();
        let _ = attributes.insert("coordinates".to_string(), json!([x, y]));
        Geometry {
            r#type: "Point".to_string(),
            attributes,
        }
    }
}

#[cfg(feature = "geo")]
impl TryFrom<Geometry> for geo::Geometry {
    type Error = Error;
    fn try_from(geometry: Geometry) -> Result<geo::Geometry> {
        serde_json::from_value::<geojson::Geometry>(serde_json::to_value(geometry)?)?
            .try_into()
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "geo")]
    fn point() {
        let point = super::Geometry::point(-108.0, 42.0);
        let geometry: geo::Geometry = point.try_into().unwrap();
        assert_eq!(
            geometry,
            geo::Geometry::Point(geo::Point::new(-108.0, 42.0))
        );
    }
}
