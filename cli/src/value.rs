use crate::{Error, Result};
use serde::Serialize;
use stac::io::{Format, IntoFormattedBytes};

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

impl IntoFormattedBytes for Value {
    fn into_formatted_bytes(self, format: Format) -> stac::Result<Vec<u8>> {
        match self {
            Self::Json(value) => value.into_formatted_bytes(format),
            Self::Stac(value) => value.into_formatted_bytes(format),
        }
    }
}
