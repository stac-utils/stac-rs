use serde::{Deserialize, Serialize};
use std::{convert::Infallible, fmt::Display, str::FromStr};

/// A version of the STAC specification.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd)]
#[allow(non_camel_case_types)]
#[non_exhaustive]
pub enum Version {
    /// [v1.0.0](https://github.com/radiantearth/stac-spec/releases/tag/v1.0.0)
    #[serde(rename = "1.0.0")]
    v1_0_0,

    /// [v1.1.0-beta.1](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0-beta.1)
    #[serde(rename = "1.1.0-beta.1")]
    v1_1_0_beta_1,

    /// [v1.1.0](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0)
    #[serde(rename = "1.1.0")]
    v1_1_0,

    /// An unknown STAC version.
    #[serde(untagged)]
    Unknown(String),
}

impl FromStr for Version {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0.0" => Ok(Version::v1_0_0),
            "1.1.0-beta.1" => Ok(Version::v1_1_0_beta_1),
            "1.1.0" => Ok(Version::v1_1_0),
            _ => Ok(Version::Unknown(s.to_string())),
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Version::v1_0_0 => "1.0.0",
                Version::v1_1_0_beta_1 => "1.1.0-beta.1",
                Version::v1_1_0 => "1.1.0",
                Version::Unknown(v) => v,
            }
        )
    }
}

impl Default for Version {
    fn default() -> Self {
        crate::STAC_VERSION
    }
}
