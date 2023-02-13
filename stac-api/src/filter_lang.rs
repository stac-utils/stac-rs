use serde::{Deserialize, Serialize};

/// The language of the filter expression.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum FilterLang {
    /// `cql2-text`
    #[serde(rename = "cql2-text")]
    Cql2Text,

    /// `cql2-json`
    #[serde(rename = "cql2-text")]
    Cql2Json,
}
