use crate::{Error, Result};
use serde::Serialize;
use stac::{IntoGeoparquet, ToNdjson};

/// An output value, which can either be a [serde_json::Value] or a [stac::Value].
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Value {
    /// A STAC value.
    Stac(stac::Value),

    /// A JSON value.
    Json(serde_json::Value),
}

impl From<stac::Value> for Value {
    fn from(value: stac::Value) -> Self {
        Value::Stac(value)
    }
}

impl From<stac_api::Item> for Value {
    fn from(value: stac_api::Item) -> Self {
        Self::Json(value.into())
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        Self::Json(value)
    }
}

impl TryFrom<Value> for stac::Value {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Stac(value) => Ok(value),
            Value::Json(value) => serde_json::from_value(value).map_err(Error::from),
        }
    }
}

impl ToNdjson for Value {
    fn to_ndjson_vec(&self) -> stac::Result<Vec<u8>> {
        match self {
            Value::Json(json) => json.to_ndjson_vec(),
            Value::Stac(stac) => stac.to_ndjson_vec(),
        }
    }

    fn to_ndjson_writer(&self, writer: impl std::io::Write) -> stac::Result<()> {
        match self {
            Value::Json(json) => json.to_ndjson_writer(writer),
            Value::Stac(stac) => stac.to_ndjson_writer(writer),
        }
    }
}

impl IntoGeoparquet for Value {
    fn into_geoparquet_writer(
        self,
        writer: impl std::io::Write + Send,
        compression: Option<stac::geoparquet::Compression>,
    ) -> stac::Result<()> {
        match self {
            Value::Json(json) => json.into_geoparquet_writer(writer, compression),
            Value::Stac(stac) => stac.into_geoparquet_writer(writer, compression),
        }
    }
}
