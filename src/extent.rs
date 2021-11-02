use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Extent {
    pub spatial: SpatialExtent,
    pub temporal: TemporalExtent,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct SpatialExtent {
    bbox: Vec<Vec<f64>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct TemporalExtent {
    interval: Vec<[Option<String>; 2]>,
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
    }
}
