use crate::{Error, Result};
use serde::{de::DeserializeOwned, Serialize};
use stac::Href;
use std::path::Path;
use url::Url;

/// Reads a STAC value from an href.
///
/// The href can be a url or a filesystem path.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// let item: stac::Item = stac_async::read("examples/simple-item.json").await.unwrap();
/// # })
/// ```
pub async fn read<T>(href: impl ToString) -> Result<T>
where
    T: DeserializeOwned + Href,
{
    let href = href.to_string();
    let mut value: T = read_json(&href).await?;
    value.set_href(href);
    Ok(value)
}

/// Reads any deserializable value from an href.
///
/// The href can be a url or a filesystem path.
///
/// # Examples
///
/// ```
/// # tokio_test::block_on(async {
/// let item: stac::Item = stac_async::read_json("examples/simple-item.json").await.unwrap();
/// # })
/// ```
pub async fn read_json<T>(href: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    if let Some(url) = stac::href_to_url(href) {
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
pub async fn write_json_to_path(path: impl AsRef<Path>, value: impl Serialize) -> Result<()> {
    let string = serde_json::to_string_pretty(&value)?;
    tokio::fs::write(path, string).await.map_err(Error::from)
}

async fn read_json_from_url<T>(url: Url) -> Result<T>
where
    T: DeserializeOwned,
{
    let response = reqwest::get(url).await?;
    response.json().await.map_err(Error::from)
}

async fn read_json_from_path<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: DeserializeOwned,
{
    let string = tokio::fs::read_to_string(path).await?;
    serde_json::from_str(&string).map_err(Error::from)
}

#[cfg(test)]
mod tests {
    use stac::{Href, Item};

    #[tokio::test]
    async fn read_filesystem() {
        let item: Item = super::read("examples/simple-item.json").await.unwrap();
        assert!(item.href().unwrap().ends_with("examples/simple-item.json"));
    }

    #[tokio::test]
    async fn read_network() {
        let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
        let item: Item = super::read(href).await.unwrap();
        assert_eq!(item.href().unwrap(), href);
    }
}
