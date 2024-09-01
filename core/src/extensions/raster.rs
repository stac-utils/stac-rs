//! The [Raster](https://github.com/stac-extensions/raster) extnesion.
//!
//! An item can describe assets that are rasters of one or multiple bands with
//! some information common to them all (raster size, projection) and also
//! specific to each of them (data type, unit, number of bits used, nodata). A
//! raster is often strongly linked with the georeferencing transform and
//! coordinate system definition of all bands (using the
//! [projection](https://github.com/stac-extensions/projection) extension).  In
//! many applications, it is interesting to have some metadata about the rasters
//! in the asset (values statistics, value interpretation, transforms).

use super::Extension;
pub use crate::{DataType, Statistics};
use serde::{Deserialize, Serialize};

/// The raster extension.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Raster {
    /// An array of available bands where each object is a [Band].
    ///
    /// If given, requires at least one band.
    pub bands: Vec<Band>,
}

/// The bands of a raster asset.
#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Band {
    /// Pixel values used to identify pixels that are nodata in the band either
    /// by the pixel value as a number or nan, inf or -inf (all strings).
    ///
    /// The extension specifies that this can be a number or a string, but we
    /// just use a f64 with a custom (de)serializer.
    ///
    /// TODO write custom (de)serializer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nodata: Option<f64>,

    /// One of area or point.
    ///
    /// Indicates whether a pixel value should be assumed to represent a
    /// sampling over the region of the pixel or a point sample at the center of
    /// the pixel.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sampling: Option<Sampling>,

    /// The data type of the pixels in the band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data_type: Option<DataType>,

    /// The actual number of bits used for this band.
    ///
    /// Normally only present when the number of bits is non-standard for the
    /// datatype, such as when a 1 bit TIFF is represented as byte.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bits_per_sample: Option<u64>,

    /// Average spatial resolution (in meters) of the pixels in the band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spatial_resolution: Option<f64>,

    /// Statistics of all the pixels in the band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub statistics: Option<Statistics>,

    /// Unit denomination of the pixel value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unit: Option<String>,

    /// Multiplicator factor of the pixel value to transform into the value
    /// (i.e. translate digital number to reflectance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,

    /// Number to be added to the pixel value (after scaling) to transform into
    /// the value (i.e. translate digital number to reflectance).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub offset: Option<f64>,

    /// Histogram distribution information of the pixels values in the band.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub histogram: Option<Histogram>,
}

/// Indicates whether a pixel value should be assumed
/// to represent a sampling over the region of the pixel or a point sample
/// at the center of the pixel.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum Sampling {
    /// The pixel value is a sampling over the region.
    Area,

    /// The pixel value is a point sample at the center of the pixel.
    Point,
}

/// The distribution of pixel values of a band can be provided with a histogram
/// object.
///
/// Those values are sampled in buckets. A histogram object is atomic and all
/// fields are REQUIRED.
#[derive(Debug, Serialize, Deserialize)]
pub struct Histogram {
    /// Number of buckets of the distribution.
    pub count: u64,

    /// Minimum value of the distribution. Also the mean value of the first bucket.
    pub min: f64,

    /// Maximum value of the distribution. Also the mean value of the last bucket.
    pub max: f64,

    /// Array of integer indicating the number of pixels included in the bucket.
    pub buckets: Vec<u64>,
}

impl Extension for Raster {
    const IDENTIFIER: &'static str = "https://stac-extensions.github.io/raster/v1.1.0/schema.json";
    const PREFIX: &'static str = "raster";
}

#[cfg(feature = "gdal")]
impl From<gdal::raster::GdalDataType> for DataType {
    fn from(value: gdal::raster::GdalDataType) -> Self {
        use gdal::raster::GdalDataType;

        match value {
            GdalDataType::Unknown => DataType::Other,
            #[cfg(gdal_has_int8)]
            GdalDataType::Int8 => DataType::Int8,
            GdalDataType::Int16 => DataType::Int16,
            GdalDataType::Int32 => DataType::Int32,
            #[cfg(gdal_has_int64)]
            GdalDataType::Int64 => DataType::Int64,
            GdalDataType::UInt8 => DataType::UInt8,
            GdalDataType::UInt16 => DataType::UInt16,
            GdalDataType::UInt32 => DataType::UInt32,
            #[cfg(gdal_has_uint64)]
            GdalDataType::UInt64 => DataType::UInt64,
            GdalDataType::Float32 => DataType::Float32,
            GdalDataType::Float64 => DataType::Float64,
        }
    }
}

#[cfg(feature = "gdal")]
impl From<gdal::raster::StatisticsAll> for Statistics {
    fn from(value: gdal::raster::StatisticsAll) -> Self {
        Statistics {
            minimum: Some(value.min),
            maximum: Some(value.max),
            mean: Some(value.mean),
            stddev: Some(value.std_dev),
            valid_percent: None,
        }
    }
}
