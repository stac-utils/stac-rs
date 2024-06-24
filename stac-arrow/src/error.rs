use arrow::datatypes::Schema;
use thiserror::Error;

/// Crate-specific error enum
#[derive(Debug, Error)]
pub enum Error {
    /// [arrow::error::ArrowError]
    #[error(transparent)]
    Arrow(#[from] arrow::error::ArrowError),

    /// Two record batches have different schemas.
    #[error("different schemas")]
    DifferentSchemas(Schema, Schema),

    /// [geoarrow::error::GeoArrowError]
    #[error(transparent)]
    GeoArrow(#[from] geoarrow::error::GeoArrowError),

    /// [geojson::Error]
    #[error(transparent)]
    Geojson(#[from] geojson::Error),

    /// [geozero::error::GeozeroError]
    #[error(transparent)]
    Geozero(#[from] geozero::error::GeozeroError),

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Invalid bbox.
    ///
    /// TODO this should probably be in the stac crate.
    #[error("invalid bbox")]
    InvalidBbox(Vec<f64>),

    /// An invalid datetime string.
    #[error("invalid datetime: {0}")]
    InvalidDatetime(String),

    /// A required field is missing.
    #[error("missing required field: {0}")]
    MissingField(&'static str),

    /// The geometry column is not binary.
    #[error("non-binary geometry column")]
    NonBinaryGeometryColumn,

    /// No items to serialize.
    #[error("no items")]
    NoItems,

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),
}
