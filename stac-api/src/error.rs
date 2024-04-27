use crate::Search;
use serde_json::{Map, Value};
use thiserror::Error;

/// Crate-specific error enum.
#[derive(Debug, Error)]
#[non_exhaustive]
pub enum Error {
    /// Queries cannot be converted to strings.
    #[error("cannot convert queries to strings")]
    CannotConvertQueryToString(Map<String, Value>),

    /// CQL2 JSON cannot (currently) be converted to strings.
    ///
    /// TODO support conversion
    #[error("cannot convert cql2-json to strings")]
    CannotConvertCql2JsonToString(Map<String, Value>),

    /// [geojson::Error]
    #[error(transparent)]
    GeoJson(#[from] geojson::Error),

    /// [std::num::ParseIntError]
    #[error(transparent)]
    ParseIntError(#[from] std::num::ParseIntError),

    /// [std::num::ParseFloatError]
    #[error(transparent)]
    ParseFloatError(#[from] std::num::ParseFloatError),

    /// A search has both bbox and intersects.
    #[error("search has bbox and intersects")]
    SearchHasBboxAndIntersects(Search),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [serde_urlencoded::ser::Error]
    #[error(transparent)]
    SerdeUrlencodedSer(#[from] serde_urlencoded::ser::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),

    /// [std::num::TryFromIntError]
    #[error(transparent)]
    TryFromInt(#[from] std::num::TryFromIntError),

    /// [url::ParseError]
    #[error(transparent)]
    UrlParse(#[from] url::ParseError),

    /// This functionality is not yet implemented.
    #[error("this functionality is not yet implemented: {0}")]
    Unimplemented(&'static str),
}
