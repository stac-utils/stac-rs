/// Crate-specific error type.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// [reqwest::Error]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),
}
