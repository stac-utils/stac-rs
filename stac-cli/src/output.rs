use std::fmt::Display;

/// CLI output.
#[derive(Debug)]
pub enum Output {
    /// A STAC value.
    Stac(stac::Value),

    /// A serde_json value.
    Json(serde_json::Value),

    /// A string.
    String(String),
}

impl Output {
    /// Converts this output to Json.
    ///
    /// Strings are not converted.
    pub fn to_json(&self) -> Option<serde_json::Value> {
        match self {
            Output::Stac(value) => serde_json::to_value(value).ok(),
            Output::Json(value) => Some(value.clone()),
            Output::String(_) => None,
        }
    }

    /// Converts this output to [stac::Value].
    ///
    /// Strings are not converted.
    pub fn to_stac(&self) -> Option<stac::Value> {
        match self {
            Output::Stac(value) => Some(value.clone()),
            Output::Json(value) => serde_json::from_value(value.clone()).ok(),
            Output::String(_) => None,
        }
    }
}

impl From<stac::Value> for Output {
    fn from(value: stac::Value) -> Self {
        Output::Stac(value)
    }
}

impl From<serde_json::Value> for Output {
    fn from(value: serde_json::Value) -> Self {
        Output::Json(value)
    }
}

impl From<serde_json::Map<String, serde_json::Value>> for Output {
    fn from(value: serde_json::Map<String, serde_json::Value>) -> Self {
        Output::Json(serde_json::Value::Object(value))
    }
}

impl From<String> for Output {
    fn from(value: String) -> Self {
        Output::String(value)
    }
}

impl From<&str> for Output {
    fn from(value: &str) -> Self {
        Output::String(value.to_string())
    }
}

impl Display for Output {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Output::Stac(value) => write!(
                f,
                "{}",
                serde_json::to_string(value).unwrap_or_else(|err| err.to_string())
            ),
            Output::Json(value) => write!(
                f,
                "{}",
                serde_json::to_string(value).unwrap_or_else(|err| err.to_string())
            ),
            Output::String(string) => write!(f, "{}", string),
        }
    }
}
