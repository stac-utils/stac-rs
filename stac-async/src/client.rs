use crate::Error;
use reqwest::{IntoUrl, Method, StatusCode};
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
        if let Some(mut value) = self
            .request::<(), V>(Method::GET, url.clone(), None)
            .await?
        {
            value.set_href(url);
            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    /// Posts data to a url.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac_api::Search;
    /// let client = stac_async::Client::new();
    /// let href = "https://planetarycomputer.microsoft.com/api/stac/v1/search";
    /// # tokio_test::block_on(async {
    /// let items: stac_api::ItemCollection = client.post(href, &Search::new().limit(1)).await.unwrap().unwrap();
    /// # })
    /// ```
    pub async fn post<S, R>(&self, url: impl IntoUrl, data: &S) -> Result<Option<R>, Error>
    where
        S: Serialize + 'static,
        R: DeserializeOwned,
    {
        self.request(Method::POST, url, Some(data)).await
    }

    /// Sends a request to a url.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Item;
    /// use reqwest::Method;
    ///
    /// let client = stac_async::Client::new();
    /// let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
    /// # tokio_test::block_on(async {
    /// let item = client.request::<(), Item>(Method::GET, href, None).await.unwrap().unwrap();
    /// # })
    /// ```
    pub async fn request<S, R>(
        &self,
        method: Method,
        url: impl IntoUrl,
        params: impl Into<Option<&S>>,
    ) -> Result<Option<R>, Error>
    where
        S: Serialize + 'static,
        R: DeserializeOwned,
    {
        let url = url.into_url()?;
        let request = match method {
            Method::GET => {
                let mut request = self.0.get(url);
                if let Some(query) = params.into() {
                    request = request.query(query);
                }
                request
            }
            Method::POST => {
                let mut request = self.0.post(url);
                if let Some(data) = params.into() {
                    request = request.json(&data);
                }
                request
            }
            _ => unimplemented!(),
        };
        let response = request.send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let response = response.error_for_status()?;
        response.json().await.map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::Client;
    use mockito::Server;
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
    async fn client_get_404() {
        let client = Client::new();
        let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/not-an-item.json";
        assert!(client.get::<stac::Item>(href).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn client_post() {
        let mut server = Server::new_async().await;
        let page = server
            .mock("POST", "/search")
            .with_body(include_str!("../mocks/page-1.json"))
            .with_header("content-type", "application/geo+json")
            .create_async()
            .await;
        let client = Client::new();
        let href = format!("{}/search", server.url());
        let _: stac_api::ItemCollection = client
            .post(href, &Search::new().limit(1))
            .await
            .unwrap()
            .unwrap();
        page.assert_async().await;
    }
}
