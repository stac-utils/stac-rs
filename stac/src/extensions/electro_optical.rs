//! The [electro-optical](https://github.com/stac-extensions/eo) extension.

use crate::Extension;
use serde::{Deserialize, Serialize};

/// EO data is considered to be data that represents a snapshot of the Earth for
/// a single date and time.
///
/// It could consist of multiple spectral bands in any part of the
/// electromagnetic spectrum. Examples of EO data include sensors with visible,
/// short-wave and mid-wave IR bands (e.g., the OLI instrument on Landsat-8),
/// long-wave IR bands (e.g. TIRS aboard Landsat-8).
#[derive(Debug, Serialize, Deserialize)]
pub struct ElectroOptical {
    /// An array of available bands where each object is a [Band].
    ///
    /// If given, requires at least one band.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub bands: Vec<Band>,

    /// Estimate of cloud cover, in %.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cloud_cover: Option<f64>,

    /// Estimate of snow and ice cover, in %.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snow_cover: Option<f64>,
}

/// [Spectral
/// bands](https://www.sciencedirect.com/topics/earth-and-planetary-sciences/spectral-band)
/// in an [Asset](crate::Asset).
#[derive(Debug, Serialize, Deserialize)]
pub struct Band {
    /// The name of the band (e.g., "B01", "B8", "band2", "red").
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// The name commonly used to refer to the band to make it easier to search for bands across instruments.
    ///
    /// See the list of [accepted common names](https://github.com/stac-extensions/eo#common-band-names).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub common_name: Option<String>,

    /// Description to fully explain the band.
    ///
    /// [CommonMark 0.29](http://commonmark.org/) syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// The center wavelength of the band, in micrometers (μm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center_wavelength: Option<f64>,

    /// Full width at half maximum (FWHM).
    ///
    /// The width of the band, as measured at half the maximum transmission, in
    /// micrometers (μm).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub full_width_half_max: Option<f64>,

    /// The solar illumination of the band, as measured at half the maximum transmission, in W/m2/micrometers.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub solar_illumination: Option<f64>,
}

impl Extension for ElectroOptical {
    const IDENTIFIER: &'static str = "https://stac-extensions.github.io/eo/v1.1.0/schema.json";
    const PREFIX: &'static str = "eo";
}

#[cfg(test)]
mod tests {
    use super::ElectroOptical;
    use crate::{Extensions, Item};

    #[test]
    fn item() {
        let item: Item = crate::read("examples/eo/item.json").unwrap();
        let _: ElectroOptical = item.extension().unwrap().unwrap();
    }
}
