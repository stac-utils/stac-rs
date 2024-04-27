use geojson::{Geometry, Value};

/// Two-dimensional bounds.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Bounds {
    /// Minimum x value.
    pub xmin: f64,
    /// Minimum y value.
    pub ymin: f64,
    /// Maximum x value.
    pub xmax: f64,
    /// Maximum y value.
    pub ymax: f64,
}

impl Bounds {
    /// Creates a new bounds object.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Bounds;
    /// let bounds = Bounds::new(1., 2., 3., 4.);
    /// ```
    pub fn new(xmin: f64, ymin: f64, xmax: f64, ymax: f64) -> Bounds {
        Bounds {
            xmin,
            ymin,
            xmax,
            ymax,
        }
    }

    /// Returns true if the minimum bound values are smaller than the maximum.
    ///
    /// This doesn't currently handle antimeridian-crossing bounds.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Bounds;
    /// let bounds = Bounds::default();
    /// assert!(!bounds.is_valid());
    /// let bounds = Bounds::new(1., 2., 3., 4.);
    /// assert!(bounds.is_valid());
    /// ```
    pub fn is_valid(&self) -> bool {
        self.xmin < self.xmax && self.ymin < self.ymax
    }

    /// Updates these bounds with another bounds' values.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Bounds;
    /// let mut bounds = Bounds::new(1., 1., 2., 2.);
    /// bounds.update(Bounds::new(0., 0., 1.5, 1.5));
    /// assert_eq!(bounds, Bounds::new(0., 0., 2., 2.));
    /// ```
    pub fn update(&mut self, other: Bounds) {
        self.xmin = self.xmin.min(other.xmin);
        self.ymin = self.ymin.min(other.ymin);
        self.xmax = self.xmax.max(other.xmax);
        self.ymax = self.ymax.max(other.ymax);
    }

    /// Converts this bounds to a [Geometry](geojson::Geometry).
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Bounds;
    /// let bounds = Bounds::new(1., 1., 2., 2.);
    /// let geomtry = bounds.to_geometry();
    /// ```
    pub fn to_geometry(&self) -> Geometry {
        Geometry {
            bbox: None,
            value: Value::Polygon(vec![vec![
                vec![self.xmin, self.ymin],
                vec![self.xmax, self.ymin],
                vec![self.xmax, self.ymax],
                vec![self.xmax, self.ymax],
                vec![self.xmin, self.ymin],
            ]]),
            foreign_members: None,
        }
    }
}

impl Default for Bounds {
    fn default() -> Self {
        Bounds {
            xmin: f64::MAX,
            ymin: f64::MAX,
            xmax: f64::MIN,
            ymax: f64::MIN,
        }
    }
}
