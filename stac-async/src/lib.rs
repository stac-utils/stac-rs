//! Asynchronous I/O for [STAC](https://stacspec.org/), built on [stac-rs](https://github.com/gadomski/stac-rs)
//!
//! This is just a small library that provides an async version of [stac::read].
//! It also includes a thin wrapper around [reqwest::Client] for efficiently
//! getting multiple STAC items in the same session.
//!
//! # Examples
//!
//! ```
//! # tokio_test::block_on(async {
//! let value = stac_async::read("data/simple-item.json").await.unwrap();
//! # })
//! ```

#[deny(missing_docs, missing_debug_implementations)]
use reqwest::IntoUrl;
use serde::Serialize;
use stac::{Href, Value};
use std::path::Path;
use url::Url;

/// Crate-specific error type. Just a wrapper around a couple of different
/// errors that could occur.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// [std::io::Error]
    #[error(transparent)]
    Io(#[from] std::io::Error),

    /// [reqwest::Error]
    #[error(transparent)]
    Reqwest(#[from] reqwest::Error),

    /// [serde_json::Error]
    #[error(transparent)]
    SerdeJson(#[from] serde_json::Error),

    /// [stac::Error]
    #[error(transparent)]
    Stac(#[from] stac::Error),
}

/// A thin wrapper around [reqwest::Client].
#[derive(Clone, Debug)]
pub struct Client(pub reqwest::Client);

impl Client {
    /// Creates a new client.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = stac_async::Client::new();
    /// ```
    ///
    /// ## Custom client
    ///
    /// You can construct the client directly using a pre-built
    /// [reqwest::Client], e.g. to do authorization:
    ///
    /// ```
    /// use reqwest::header;
    /// let mut headers = header::HeaderMap::new();
    /// let mut auth_value = header::HeaderValue::from_static("secret");
    /// auth_value.set_sensitive(true);
    /// headers.insert(header::AUTHORIZATION, auth_value);
    /// let client = reqwest::Client::builder().default_headers(headers).build().unwrap();
    /// let client = stac_async::Client(client);
    /// ```
    pub fn new() -> Client {
        Client(reqwest::Client::new())
    }

    /// Gets a STAC value from a url.
    ///
    /// Also sets that [Values](Value) href.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = stac_async::Client::new();
    /// let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
    /// # tokio_test::block_on(async {
    /// let value = client.get(href).await.unwrap();
    /// # })
    /// ```
    pub async fn get(&self, url: impl IntoUrl) -> Result<Value, Error> {
        let url = url.into_url()?;
        let response = self.0.get(url.clone()).send().await?;
        let value: serde_json::Value = response.json().await?;
        let mut value = Value::from_json(value)?;
        value.set_href(url);
        Ok(value)
    }
}

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
    use super::Client;
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

    #[tokio::test]
    async fn client() {
        let client = Client::new();
        let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
        let value = client.get(href).await.unwrap();
        assert_eq!(value.href().unwrap(), href);
    }
}
