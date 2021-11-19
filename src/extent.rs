use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The object describes the spatio-temporal extents of the Collection.
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Extent {
    /// Potential spatial extents covered by the Collection.
    pub spatial: SpatialExtent,
    /// Potential temporal extents covered by the Collection.
    pub temporal: TemporalExtent,

    /// Additional fields on the extent.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

/// The object describes the spatial extents of the Collection.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct SpatialExtent {
    /// Potential spatial extents covered by the Collection.
    pub bbox: Vec<Vec<f64>>,
}

/// The object describes the temporal extents of the Collection.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct TemporalExtent {
    /// Potential temporal extents covered by the Collection.
    pub interval: Vec<[Option<String>; 2]>,
}

impl Default for SpatialExtent {
    fn default() -> SpatialExtent {
        SpatialExtent {
            bbox: vec![vec![-180.0, -90.0, 180.0, 90.0]],
        }
    }
}

impl Default for TemporalExtent {
    fn default() -> TemporalExtent {
        TemporalExtent {
            interval: vec![[None, None]],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Extent;

    #[test]
    fn default() {
        let extent = Extent::default();
        assert_eq!(extent.spatial.bbox, [[-180.0, -90.0, 180.0, 90.0]]);
        assert_eq!(extent.temporal.interval, [[None, None]]);
        assert!(extent.additional_fields.is_empty());
    }
}
