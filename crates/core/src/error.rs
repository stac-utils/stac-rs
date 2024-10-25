use crate::Version;
#[cfg(feature = "validate")]
use jsonschema::ValidationError;
use thiserror::Error;

/// Error enum for crate-specific errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    /// [arrow_schema::ArrowError]
    #[error(transparent)]
    #[cfg(feature = "geoarrow")]
    Arrow(#[from] arrow_schema::ArrowError),

    /// [chrono::ParseError]
    #[error(transparent)]
    ChronoParse(#[from] chrono::ParseError),

    /// A required feature is not enabled.
    #[error("{0} is not enabled")]
    FeatureNotEnabled(&'static str),

    /// [fluent_uri::error::ParseError]
    #[error(transparent)]
    #[cfg(feature = "validate")]
    FluentUriParse(#[from] fluent_uri::error::ParseError),

    /// [geoarrow::error::GeoArrowError]
    #[error(transparent)]
    #[cfg(feature = "geoarrow")]
    GeoArrow(#[from] geoarrow::error::GeoArrowError),

    /// [geojson::Error]
    #[error(transparent)]
    Geojson(#[from] Box<geojson::Error>),

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// Returned when a property name conflicts with a top-level STAC field, or
    /// it's an invalid top-level field name.
    #[error("invalid attribute name: {0}")]
    InvalidAttribute(String),

    /// Returned when a STAC object has the wrong type field.
    #[error("incorrect type: expected={expected}, actual={actual}")]
    IncorrectType {
        /// The actual type field on the object.
        actual: String,

        /// The expected value.
        expected: String,
    },

    /// This vector is not a valid bounding box.
    #[error("invalid bbox: {0:?}")]
    InvalidBbox(Vec<f64>),

    /// This string is not a valid datetime interval.
    #[error("invalid datetime: {0}")]
    InvalidDatetime(String),

    /// Returned when there is not a required field on a STAC object
    #[error("no \"{0}\" field in the JSON object")]
    MissingField(&'static str),

    /// There is not an href, when an href is required.
    #[error("no href")]
    NoHref,

    /// There are no items, when items are required.
    #[error("no items")]
    NoItems,

    /// This is not a JSON object.
    #[error("json value is not an object")]
    NotAnObject(serde_json::Value),

    /// [object_store::Error]
    #[error(transparent)]
    #[cfg(feature = "object-store")]
    ObjectStore(#[from] object_store::Error),

    /// [object_store::path::Error]
    #[error(transparent)]
    #[cfg(feature = "object-store")]
    ObjectStorePath(#[from] object_store::path::Error),

    /// [parquet::errors::ParquetError]
    #[error(transparent)]
    #[cfg(feature = "geoparquet")]
    Parquet(#[from] parquet::errors::ParquetError),

    /// [reqwest::Error]
    #[cfg(any(feature = "reqwest", feature = "validate"))]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// JSON is a scalar when an array or object was expected
    #[error("json value is not an object or an array")]
    ScalarJson(serde_json::Value),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [tokio::task::JoinError]
    #[error(transparent)]
    #[cfg(feature = "validate")]
    TokioJoin(#[from] tokio::task::JoinError),

    /// [std::num::TryFromIntError]
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),

    /// Returned when the `type` field of a STAC object does not equal `"Feature"`, `"Catalog"`, or `"Collection"`.
    #[error("unknown \"type\": {0}")]
    UnknownType(String),

    /// Unsupported migration.
    #[error("unsupported migration: {0} to {1}")]
    UnsupportedMigration(Version, Version),

    /// Unsupported file format.
    #[error("unsupported format: {0}")]
    UnsupportedFormat(String),

    /// Unsupported geoparquet type
    #[error("unsupported geoparquet type")]
    UnsupportedGeoparquetType,

    /// [url::ParseError]
    #[error(transparent)]
    Url(#[from] url::ParseError),

    /// A list of validation errors.
    ///
    /// Since we usually don't have the original [serde_json::Value] (because we
    /// create them from the STAC objects), we need these errors to be `'static`
    /// lifetime.
    #[error("validation errors")]
    #[cfg(feature = "validate")]
    Validation(Vec<ValidationError<'static>>),
}

#[cfg(feature = "validate")]
impl Error {
    pub(crate) fn from_validation_errors<'a, I>(errors: I) -> Error
    where
        I: Iterator<Item = ValidationError<'a>>,
    {
        use std::borrow::Cow;

        let mut error_vec = Vec::new();
        for error in errors {
            // Cribbed from https://docs.rs/jsonschema/latest/src/jsonschema/error.rs.html#21-30
            error_vec.push(ValidationError {
                instance_path: error.instance_path.clone(),
                instance: Cow::Owned(error.instance.into_owned()),
                kind: error.kind,
                schema_path: error.schema_path,
            })
        }
        Error::Validation(error_vec)
    }
}
