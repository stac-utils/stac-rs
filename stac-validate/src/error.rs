use jsonschema::ValidationError;
use std::borrow::Cow;
use thiserror::Error;
use url::Url;

/// Crate-specific error type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Cannot resolve schemas with a json-schema scheme.
    #[error("cannot resolve json-schema scheme: {0}")]
    CannotResolveJsonSchemaScheme(Url),

    /// Missing stac_version.
    #[error("missing stac_version attribute")]
    MissingStacVersion,

    /// The `stac_extensions` vector, or its contents, are not the correct type.
    #[error("incorrect stac extensions type")]
    IncorrectStacExtensionsType(serde_json::Value),

    /// The url is not a valid file path.
    #[error("invalid file path: {0}")]
    InvalidFilePath(Url),

    /// We cannot handle this url scheme.
    #[error("invalid url scheme: {0}")]
    InvalidUrlScheme(Url),

    /// Invalid JSONSchema.
    #[error("invalid json-schema at url: {0}")]
    InvalidSchema(String),

    /// [reqwest::Error]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// A list of validation errors.
    ///
    /// Since we usually don't have the original [serde_json::Value] (because we
    /// create them from the STAC objects), we need these errors to be `'static`
    /// lifetime.
    #[error("validation errors")]
    Validation(Vec<ValidationError<'static>>),

    /// [jsonschema::ValidationError]
    #[error(transparent)]
    JSONSchemaValidation(#[from] ValidationError<'static>),
}

impl Error {
    /// Creates an [crate::Error] from an iterator over [jsonschema::ValidationError].
    #[allow(single_use_lifetimes)]
    pub fn from_validation_errors<'a>(errors: impl Iterator<Item = ValidationError<'a>>) -> Error {
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

    /// Creates an [crate::Error] from a single [jsonschema::ValidationError].
    pub fn from_validation_error(error: ValidationError<'_>) -> Error {
        // Cribbed from https://docs.rs/jsonschema/latest/src/jsonschema/error.rs.html#21-30
        Error::JSONSchemaValidation(ValidationError {
            instance_path: error.instance_path.clone(),
            instance: Cow::Owned(error.instance.into_owned()),
            kind: error.kind,
            schema_path: error.schema_path,
        })
    }
}
