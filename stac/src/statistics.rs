use serde::{Deserialize, Serialize};

/// Statistics of all pixels in the band.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Statistics {
    /// Mean value of all the pixels in the band
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mean: Option<f64>,

    /// Minimum value of all the pixels in the band
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minimum: Option<f64>,

    /// Maximum value of all the pixels in the band
    #[serde(skip_serializing_if = "Option::is_none")]
    pub maximum: Option<f64>,

    /// Standard deviation value of all the pixels in the band
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stddev: Option<f64>,

    /// Percentage of valid (not nodata) pixel
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid_percent: Option<f64>,
}
