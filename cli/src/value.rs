use serde::Serialize;

/// An output value, which can either be a [serde_json::Value], a [stac::Value], or a [String].
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
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Stac(value) => Ok(value),
            Value::Json(value) => serde_json::from_value(value),
        }
    }
}
