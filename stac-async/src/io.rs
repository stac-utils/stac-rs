use crate::Error;
use serde::Serialize;
use stac::{Href, Value};
use std::path::Path;
use url::Url;

/// Reads a STAC [Value] from an href.
///
/// The href can be a url or a filesystem path.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// let value = stac_async::read("data/simple-item.json").await.unwrap();
/// # })
/// ```
pub async fn read(href: impl ToString) -> Result<Value, Error> {
    let href = href.to_string();
    let value = read_json(&href).await?;
    let mut value = Value::from_json(value)?;
    value.set_href(href);
    Ok(value)
}

/// Reads a [serde_json::Value] from an href.
///
/// The href can be a url or a filesystem path.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// let value = stac_async::read_json("data/simple-item.json").await.unwrap();
/// # })
/// ```
pub async fn read_json(href: &str) -> Result<serde_json::Value, Error> {
    if let Ok(url) = Url::parse(&href) {
        read_json_from_url(url).await
    } else {
        read_json_from_path(href).await
    }
}

/// Writes any serializable value to a path.
///
/// # Examples
///
/// ```no_run
/// let item = stac::Item::new("an-id");
/// # tokio_test::block_on(async {
/// let value = stac_async::write_json_to_path("item.json", item).await.unwrap();
/// # })
/// ```
pub async fn write_json_to_path(
    path: impl AsRef<Path>,
    value: impl Serialize,
) -> Result<(), Error> {
    let string = serde_json::to_string_pretty(&value)?;
    tokio::fs::write(path, string).await.map_err(Error::from)
}

async fn read_json_from_url(url: Url) -> Result<serde_json::Value, Error> {
    let response = reqwest::get(url).await?;
    response.json().await.map_err(Error::from)
}

async fn read_json_from_path(path: impl AsRef<Path>) -> Result<serde_json::Value, Error> {
    let string = tokio::fs::read_to_string(path).await?;
    serde_json::from_str(&string).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use stac::Href;

    #[tokio::test]
    async fn read_filesystem() {
        let value = super::read("data/simple-item.json").await.unwrap();
        assert!(value.href().unwrap().ends_with("data/simple-item.json"));
    }

    #[tokio::test]
    async fn read_network() {
        let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
        let value = super::read(href).await.unwrap();
        assert_eq!(value.href().unwrap(), href);
    }
}
