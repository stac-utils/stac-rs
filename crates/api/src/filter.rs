use std::{convert::Infallible, str::FromStr};

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The language of the filter expression.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "filter-lang", content = "filter")]
pub enum Filter {
    /// `cql2-text`
    #[serde(rename = "cql2-text")]
    Cql2Text(String),

    /// `cql2-json`
    #[serde(rename = "cql2-json")]
    Cql2Json(Map<String, Value>),
}

impl Default for Filter {
    fn default() -> Self {
        Filter::Cql2Json(Default::default())
    }
}

impl FromStr for Filter {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Filter::Cql2Text(s.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::Filter;
    use serde_json::json;

    #[test]
    fn json() {
        let filter = Filter::Cql2Json(json!({
                  "filter": {
                    "op" : "and",
                    "args": [
                      {
                        "op": "=",
                        "args": [ { "property": "id" }, "LC08_L1TP_060247_20180905_20180912_01_T1_L1TP" ]
                      },
                      {
                        "op": "=",
                        "args" : [ { "property": "collection" }, "landsat8_l1tp" ]
                      }
                    ]
                  }
                }
            ).as_object().unwrap().clone(),
        );
        let value = serde_json::to_value(filter).unwrap();
        assert_eq!(value["filter-lang"], "cql2-json");
        assert!(value.get("filter").is_some());
    }

    #[test]
    fn text() {
        let filter = Filter::Cql2Text(
            "id='LC08_L1TP_060247_20180905_20180912_01_T1_L1TP' AND collection='landsat8_l1tp'"
                .to_string(),
        );
        let value = serde_json::to_value(filter).unwrap();
        assert_eq!(value["filter-lang"], "cql2-text");
        assert!(value.get("filter").is_some());
    }
}
