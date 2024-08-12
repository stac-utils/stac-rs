use crate::Error;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

/// A version of the STAC specification.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize, Eq, Hash, PartialOrd)]
#[allow(non_camel_case_types)]
pub enum Version {
    /// [v1.0.0](https://github.com/radiantearth/stac-spec/releases/tag/v1.0.0)
    #[serde(rename = "1.0.0")]
    v1_0_0,

    /// [v1.1.0-beta.1](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0-beta.1)
    #[serde(rename = "1.1.0-beta.1")]
    v1_1_0_beta_1,
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "1.0.0" => Ok(Version::v1_0_0),
            "1.1.0-beta.1" => Ok(Version::v1_1_0_beta_1),
            _ => Err(Error::UnsupportedVersion(s.to_string())),
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
            }
        )
    }
}
