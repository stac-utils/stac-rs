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

use serde::{Deserialize, Serialize};

use super::Extension;

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
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sampling {
    /// The pixel value is a sampling over the region.
    Area,

    /// The pixel value is a point sample at the center of the pixel.
    Point,
}

/// The data type gives information about the values in the file.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DataType {
    /// 8-bit integer
    Int8,

    /// 16-bit integer
    Int16,

    /// 32-bit integer
    Int32,

    /// 64-bit integer
    Int64,

    /// Unsigned 8-bit integer (common for 8-bit RGB PNG's)
    UInt8,

    /// Unsigned 16-bit integer
    UInt16,

    /// Unsigned 32-bit integer
    UInt32,

    /// Unsigned 64-bit integer
    UInt64,

    /// 16-bit float
    Float16,

    /// 32-bit float
    Float32,

    /// 64-bit float
    Float64,

    /// 16-bit complex integer
    CInt16,

    /// 32-bit complex integer
    CInt32,

    /// 32-bit complex float
    CFloat32,

    /// 64-bit complex float
    CFloat64,

    /// Other data type than the ones listed above (e.g. boolean, string, higher precision numbers)
    Other,
}

/// Statistics of all pixels in the band.
#[derive(Debug, Serialize, Deserialize)]
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
