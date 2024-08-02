//! Input/output utilities.

use crate::{Error, Result};
use serde::de::DeserializeOwned;

/// Reads something from an href or from standard input.
pub async fn read_href<D: DeserializeOwned>(href: Option<&str>) -> Result<D> {
    // TODO support `-` for stdin
    if let Some(href) = href {
        stac_async::read_json(href).await.map_err(Error::from)
    } else {
        serde_json::from_reader(std::io::stdin()).map_err(Error::from)
    }
}
