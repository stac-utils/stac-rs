use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

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
        let mut attributes = Map::new();
        let _ = attributes.insert("coordinates".to_string(), json!([x, y]));
        Geometry {
            r#type: "Point".to_string(),
            attributes,
        }
    }

    /// Creates a geometry.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Geometry;
    /// let geometry = Geometry::rect(-108.0, 42.0, -107.0, 43.0);
    /// ```
    pub fn rect(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Geometry {
        let mut attributes = Map::new();
        let _ = attributes.insert(
            "coordinates".to_string(),
            json!([[
                [xmin, ymin],
                [xmax, ymin],
                [xmax, ymax],
                [xmin, ymax],
                [xmin, ymin]
            ]]),
        );
        Geometry {
            r#type: "Polygon".to_string(),
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

#[cfg(all(test, feature = "geo"))]
mod tests {
    use geo::{algorithm::orient::Direction, Orient, Point, Polygon, Rect};

    #[test]
    fn point() {
        let point = super::Geometry::point(-108.0, 42.0);
        let geometry: geo::Geometry = point.try_into().unwrap();
        assert_eq!(geometry, geo::Geometry::Point(Point::new(-108.0, 42.0)));
    }

    #[test]
    fn rect() {
        let rect = super::Geometry::rect(-108.0, 42.0, -107.0, 43.0);
        let geometry: geo::Geometry = rect.try_into().unwrap();
        assert_eq!(
            Polygon::try_from(geometry).unwrap(),
            Rect::new(
                geo::coord! { x: -108.0, y: 42.0 },
                geo::coord! { x: -107.0, y: 43.0 },
            )
            .to_polygon()
            .orient(Direction::Default)
        );
    }
}
