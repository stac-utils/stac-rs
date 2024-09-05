use serde::Serialize;

/// An output value, which can either be a [serde_json::Value], a [stac::Value], or a [String].
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Value {
    /// A STAC value.
    Stac(stac::Value),

    /// A JSON value.
    Json(serde_json::Value),

    /// A string value.
    String(String),
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

impl From<String> for Value {
    fn from(value: String) -> Self {
        Self::String(value)
    }
}

impl TryFrom<Value> for stac::Value {
    type Error = serde_json::Error;
    fn try_from(value: Value) -> Result<Self, Self::Error> {
        match value {
            Value::Stac(value) => Ok(value),
            Value::Json(value) => serde_json::from_value(value),
            Value::String(string) => serde_json::from_str(&string),
        }
    }
}
