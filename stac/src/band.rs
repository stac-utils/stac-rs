use crate::{DataType, Fields, Statistics};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// An array of bands is used to describe the available bands in a STAC entity
/// or Asset.
///
/// These fields describes the general construct of a band or layer, which
/// doesn't necessarily need to be a spectral band. By adding fields from
/// extensions you can indicate that a band, for example, is
#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Clone)]
pub struct Band {
    /// The name of the band (e.g., "B01", "B8", "band2", "red"), which should
    /// be unique across all bands defined in the list of bands.
    ///
    /// This is typically the name the data provider uses for the band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,

    /// Description to fully explain the band.
    ///
    /// CommonMark 0.29 syntax MAY be used for rich text representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,

    /// Pixel values used to identify pixels that are nodata in the band either
    /// by the pixel value as a number or nan, inf or -inf (all strings).
    ///
    /// The extension specifies that this can be a number or a string, but we
    /// just use a f64 with a custom (de)serializer.
    ///
    /// TODO write custom (de)serializer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodata: Option<f64>,

    /// The data type of the pixels in the band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,

    /// Statistics of all the pixels in the band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<Statistics>,

    /// Unit denomination of the pixel value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Additional fields on the band.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl Fields for Band {
    fn fields(&self) -> &Map<String, Value> {
        &self.additional_fields
    }
    fn fields_mut(&mut self) -> &mut Map<String, Value> {
        &mut self.additional_fields
    }
}
