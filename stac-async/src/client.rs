use crate::Error;
use reqwest::{IntoUrl, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use stac::Href;

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
    /// Also sets that [Values](Value) href. Returns Ok(None) if a 404 is
    /// returned from the server.
    ///
    /// # Examples
    ///
    /// ```
    /// let client = stac_async::Client::new();
    /// let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
    /// # tokio_test::block_on(async {
    /// let item: stac::Item = client.get(href).await.unwrap().unwrap();
    /// # })
    /// ```
    pub async fn get<V>(&self, url: impl IntoUrl) -> Result<Option<V>, Error>
    where
        V: DeserializeOwned + Href,
    {
        let url = url.into_url()?;
        let response = self.0.get(url.clone()).send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let mut value: V = response.json().await?;
        value.set_href(url);
        Ok(Some(value))
    }

    /// Posts data to a url.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// let client = stac_async::Client::new();
    /// let href = "https://planetarycomputer.microsoft.com/api/stac/v1/search";
    /// # tokio_test::block_on(async {
    /// let items: stac_api::ItemCollection = client.post(href, &Search::new().limit(1)).await.unwrap();
    /// # })
    /// ```
    pub async fn post<S, R>(&self, url: impl IntoUrl, data: &S) -> Result<R, Error>
    where
        S: Serialize,
        R: DeserializeOwned,
    {
        let url = url.into_url()?;
        let response = self.0.post(url).json(data).send().await?;
        let response = response.error_for_status()?;
        response.json().await.map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::Client;
    use stac::{Href, Item};
    use stac_api::Search;

    #[tokio::test]
    async fn client_get() {
        let client = Client::new();
        let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
        let item: Item = client.get(href).await.unwrap().unwrap();
        assert_eq!(item.href().unwrap(), href);
    }

    #[tokio::test]
    async fn client_post() {
        let client = Client::new();
        let href = "https://planetarycomputer.microsoft.com/api/stac/v1/search";
        let _: stac_api::ItemCollection = client.post(href, &Search::new().limit(1)).await.unwrap();
    }
}
