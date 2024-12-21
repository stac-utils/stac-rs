use object_store::ObjectStoreScheme;
use thiserror::Error;

/// Error enum for crate-specific errors.
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum Error {
    #[error("Failed to create object_store for {scheme:?}. Check if required feature is enabled.")]
    ObjectStoreCreate {
        /// feature
        scheme: ObjectStoreScheme,
    },

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    /// [object_store::Error]
    #[error(transparent)]
    ObjectStore(#[from] object_store::Error),

    /// [object_store::path::Error]
    #[error(transparent)]
    ObjectStorePath(#[from] object_store::path::Error),
}
