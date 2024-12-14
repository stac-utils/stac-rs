use thiserror::Error;

/// Crate specific error type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// [gdal::errors::GdalError]
    #[error(transparent)]
    GdalError(#[from] gdal::errors::GdalError),

    /// [stac::Error]
    #[error(transparent)]
    STACError(#[from] stac::Error),

    /// [std::num::ParseIntError]
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// Failed to parse EPSG projection from proj extension.
    #[error("Failed to parse EPSG projection from: `{0}`")]
    ParseEPSGProjectionError(String),

    /// Unsupported STAC extension
    #[error("STAC extension `{0}` is not supported")]
    UnsupportedExtension(String),
}
