//! Version management for STAC.
//!
//! As the spec evolves, we'll want to be able to migrate objects from one
//! version to another.

use crate::Error;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

/// STAC versions supported by this crate.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Version {
    /// [v1.0.0](https://github.com/radiantearth/stac-spec/releases/tag/v1.0.0)
    #[serde(rename = "1.0.0")]
    v1_0_0,

    /// [v1.1.0-beta.1](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0-beta.1)
    #[serde(rename = "1.1.0-beta.1")]
    v1_1_0,
}

/// A step from one version to another.
///
/// These should not skip versions.
#[derive(Debug, Clone, Copy, PartialEq)]
#[allow(non_camel_case_types)]
pub enum Step {
    /// Migrate from 1.0.0 to 1.1.0-beta.1
    v1_0_0_to_v1_1_0,

    /// Migrate from 1.1.0-beta.1 to 1.0.0
    v1_1_0_to_v1_0_0,
}

impl Version {
    /// Returns all the steps from one version to another.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Version, version::Step};
    ///
    ///
    /// assert_eq!(
    ///     Version::v1_0_0.steps_to(Version::v1_1_0),
    ///     vec![Step::v1_0_0_to_v1_1_0]
    /// );
    /// ```
    pub fn steps_to(self, version: Version) -> Vec<Step> {
        use Version::*;

        match self {
            v1_0_0 => match version {
                v1_0_0 => Vec::new(),
                v1_1_0 => vec![Step::v1_0_0_to_v1_1_0],
            },
            v1_1_0 => match version {
                v1_0_0 => vec![Step::v1_1_0_to_v1_0_0],
                v1_1_0 => Vec::new(),
            },
        }
    }
}

impl Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        use Version::*;

        write!(
            f,
            "{}",
            match self {
                v1_0_0 => "1.0.0",
                v1_1_0 => "1.1.0-beta.1",
            }
        )
    }
}

impl FromStr for Version {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        use Version::*;

        match s {
            "1.0.0" | "v1.0.0" => Ok(v1_0_0),
            "1.1.0-beta.1" | "v1.1.0-beta.1" => Ok(v1_1_0),
            _ => Err(Error::UnsupportedVersion(s.to_string())),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Step, Version};

    #[test]
    fn steps_to() {
        assert_eq!(
            Version::v1_0_0.steps_to(Version::v1_1_0),
            vec![Step::v1_0_0_to_v1_1_0],
        );
        assert_eq!(Version::v1_0_0.steps_to(Version::v1_0_0), vec![]);
        assert_eq!(
            Version::v1_1_0.steps_to(Version::v1_0_0),
            vec![Step::v1_1_0_to_v1_0_0]
        );
    }
}
