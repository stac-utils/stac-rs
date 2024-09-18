use jsonschema::ValidationError;
use serde_json::Value;
use std::{borrow::Cow, sync::Arc};
use thiserror::Error;
use url::Url;

/// Crate-specific error type.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Cannot validate a non-object, non-array
    #[error("value is not an object or an array, cannot validate")]
    CannotValidate(Value),

    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// No type field on an object.
    #[error("no type field")]
    NoType,

    /// No version field on an object.
    #[error("no version field")]
    NoVersion,

    /// [reqwest::Error]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// [tokio::task::JoinError]
    #[error(transparent)]
    TokioJoin(#[from] tokio::task::JoinError),

    /// [tokio::sync::mpsc::error::SendError]
    #[error(transparent)]
    TokioSend(
        #[from]
        tokio::sync::mpsc::error::SendError<(
            Url,
            tokio::sync::oneshot::Sender<crate::Result<Arc<Value>>>,
        )>,
    ),

    /// [tokio::sync::oneshot::error::RecvError]
    #[error(transparent)]
    TokioRecv(#[from] tokio::sync::oneshot::error::RecvError),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    /// A list of validation errors.
    ///
    /// Since we usually don't have the original [serde_json::Value] (because we
    /// create them from the STAC objects), we need these errors to be `'static`
    /// lifetime.
    #[error("validation errors")]
    Validation(Vec<ValidationError<'static>>),
}

impl Error {
    /// Creates an [crate::Error] from an iterator over [jsonschema::ValidationError].
    pub fn from_validation_errors<'a, I>(errors: I) -> Error
    where
        I: Iterator<Item = ValidationError<'a>>,
    {
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
