use crate::Error;
use http::header::HeaderName;
use reqwest::{header::HeaderMap, IntoUrl, Method, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use serde_json::{Map, Value};
use stac::{Href, Link};

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
            .request::<(), V>(Method::GET, url.clone(), None, None)
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
    /// let mut search = Search::default();
    /// search.items.limit = Some(1);
    /// # tokio_test::block_on(async {
    /// let items: stac_api::ItemCollection = client.post(href, &search).await.unwrap().unwrap();
    /// # })
    /// ```
    pub async fn post<S, R>(&self, url: impl IntoUrl, data: &S) -> Result<Option<R>, Error>
    where
        S: Serialize + 'static,
        R: DeserializeOwned,
    {
        self.request(Method::POST, url, Some(data), None).await
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
    /// let item = client.request::<(), Item>(Method::GET, href, None, None).await.unwrap().unwrap();
    /// # })
    /// ```
    pub async fn request<S, R>(
        &self,
        method: Method,
        url: impl IntoUrl,
        params: impl Into<Option<&S>>,
        headers: impl Into<Option<HeaderMap>>,
    ) -> Result<Option<R>, Error>
    where
        S: Serialize + 'static,
        R: DeserializeOwned,
    {
        let url = url.into_url()?;
        let mut request = match method {
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
        if let Some(headers) = headers.into() {
            request = request.headers(headers);
        }
        let response = request.send().await?;
        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }
        let response = response.error_for_status()?;
        response.json().await.map_err(Error::from)
    }

    /// Builds and sends a request, as defined in a link.
    ///
    /// Used mostly for "next" links in pagination.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac::Link;
    /// let link = Link::new("http://stac-async-rs.test/search?foo=bar", "next");
    /// let client = stac_async::Client::new();
    /// # tokio_test::block_on(async {
    /// let page: stac_api::ItemCollection = client.request_from_link(link).await.unwrap().unwrap();
    /// # })
    /// ```
    pub async fn request_from_link<R>(&self, link: Link) -> Result<Option<R>, Error>
    where
        R: DeserializeOwned,
    {
        let method = if let Some(method) = link.method {
            method.parse()?
        } else {
            Method::GET
        };
        let headers = if let Some(headers) = link.headers {
            let mut header_map = HeaderMap::new();
            for (key, value) in headers.into_iter() {
                let header_name: HeaderName = key.parse()?;
                let _ = header_map.insert(header_name, value.to_string().parse()?);
            }
            Some(header_map)
        } else {
            None
        };
        self.request::<Map<String, Value>, R>(method, link.href, &link.body, headers)
            .await
    }
}

impl Default for Client {
    fn default() -> Self {
        Self::new()
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
        assert!(client.get::<Item>(href).await.unwrap().is_none());
    }

    #[tokio::test]
    async fn client_post() {
        let mut server = Server::new_async().await;
        let page = server
            .mock("POST", "/search")
            .with_body(include_str!("../mocks/search-page-1.json"))
            .with_header("content-type", "application/geo+json")
            .create_async()
            .await;
        let client = Client::new();
        let href = format!("{}/search", server.url());
        let mut search = Search::default();
        search.items.limit = Some(1);
        let _: stac_api::ItemCollection = client.post(href, &search).await.unwrap().unwrap();
        page.assert_async().await;
    }
}
