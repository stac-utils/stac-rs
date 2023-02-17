use crate::{Client, Error, Result};
use async_stream::try_stream;
use futures_core::stream::Stream;
use futures_util::{pin_mut, StreamExt};
use stac::Collection;
use stac_api::{Item, ItemCollection, Search, UrlBuilder};
use tokio::sync::mpsc;
use url::Url;

const DEFAULT_CHANNEL_BUFFER: usize = 10;

/// A client for interacting with STAC APIs.
#[derive(Debug)]
pub struct ApiClient {
    client: Client,
    channel_buffer: usize,
    url_builder: UrlBuilder,
}

impl ApiClient {
    /// Creates a new API client.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_async::ApiClient;
    /// let client = ApiClient::new("https://planetarycomputer.microsoft.com/api/stac/v1").unwrap();
    /// ```
    pub fn new(url: &str) -> Result<ApiClient> {
        Ok(ApiClient {
            client: Client::new(),
            channel_buffer: DEFAULT_CHANNEL_BUFFER,
            url_builder: UrlBuilder::new(url)?,
        })
    }

    /// Returns a single collection.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use stac_async::ApiClient;
    /// let client = ApiClient::new("https://planetarycomputer.microsoft.com/api/stac/v1").unwrap();
    /// # tokio_test::block_on(async {
    /// let collection = client.collection("sentinel-2-l2a").await.unwrap().unwrap();
    /// # })
    /// ```
    pub async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        let url = self.url_builder.collection(id)?;
        self.client.get(url).await
    }

    /// Searches an API, returning a stream of items.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac_api::Search;
    /// use stac_async::ApiClient;
    /// use futures_util::stream::StreamExt;
    ///
    /// let client = ApiClient::new("https://planetarycomputer.microsoft.com/api/stac/v1").unwrap();
    /// let search = Search::new().collection("sentinel-2-l2a").limit(1);
    /// # tokio_test::block_on(async {
    /// let items: Vec<_> = client
    ///     .search(search)
    ///     .map(|result| result.unwrap())
    ///     .collect()
    ///     .await;
    /// assert_eq!(items.len(), 1);
    /// # })
    /// ```
    pub fn search(&self, search: Search) -> impl Stream<Item = Result<Item>> {
        // TODO support GET
        let url = self.url_builder.search().clone();
        let (tx, mut rx) = mpsc::channel(self.channel_buffer);
        let client = self.client.clone();
        // TODO implement request splitting over collections
        tokio::spawn(async move {
            let pager = pager(client, url, Some(search));
            pin_mut!(pager);
            while let Some(result) = pager.next().await {
                match result {
                    Ok(page) => tx.send(Ok(page)).await.unwrap(),
                    Err(err) => {
                        tx.send(Err(err)).await.unwrap();
                        return;
                    }
                }
            }
        });
        try_stream! {
            while let Some(result) = rx.recv().await {
                let page = result?;
                for item in page.items {
                    yield item;
                }
            }
        }
    }
}

fn pager(
    client: Client,
    mut url: Url,
    mut search: Option<Search>,
) -> impl Stream<Item = Result<ItemCollection>> {
    try_stream! {
        while let Some(result) = page(client.clone(), url, search).await {
            let (page, next_url, next_search) = result?;
            yield page;
            if let Some(next_url) = next_url {
                url = next_url;
                search = next_search;
            } else {
                return;
            }
        }
    }
}

async fn page(
    client: Client,
    url: Url,
    search: Option<Search>,
) -> Option<Result<(ItemCollection, Option<Url>, Option<Search>)>> {
    // TODO support GET
    match client.post::<_, ItemCollection>(url, &search).await {
        Ok(Some(page)) => {
            if page.items.is_empty() {
                return None;
            }
            match page.next_url_and_search() {
                Ok(Some((url, search))) => Some(Ok((page, Some(url), search))),
                Ok(None) => Some(Ok((page, None, None))),
                Err(err) => Some(Err(Error::from(err))),
            }
        }
        Ok(None) => None,
        Err(err) => Some(Err(err)),
    }
}

#[cfg(test)]
mod tests {
    use super::ApiClient;
    use futures_util::stream::StreamExt;
    use mockito::{Matcher, Server};
    use serde_json::json;
    use stac::Links;
    use stac_api::{ItemCollection, Search};

    #[tokio::test]
    async fn collection_not_found() {
        let mut server = Server::new_async().await;
        let collection = server
            .mock("GET", "/collections/not-a-collection")
            .with_body(include_str!("../mocks/not-a-collection.json"))
            .with_header("content-type", "application/json")
            .with_status(404)
            .create_async()
            .await;

        let client = ApiClient::new(&server.url()).unwrap();
        assert!(client
            .collection("not-a-collection")
            .await
            .unwrap()
            .is_none());
        collection.assert_async().await;
    }

    #[tokio::test]
    async fn search_with_paging() {
        let mut server = Server::new_async().await;
        let mut page_1_body: ItemCollection =
            serde_json::from_str(include_str!("../mocks/page-1.json")).unwrap();
        let mut next_link = page_1_body.link("next").unwrap().clone();
        next_link.href = format!("{}/search", server.url());
        page_1_body.set_link(next_link);
        let page_1 = server
            .mock("POST", "/search")
            .match_body(Matcher::Json(json!({
                "collections": ["sentinel-2-l2a"],
                "limit": 1
            })))
            .with_body(serde_json::to_string(&page_1_body).unwrap())
            .with_header("content-type", "application/geo+json")
            .create_async()
            .await;
        let page_2 = server
            .mock("POST", "/search")
            .match_body(Matcher::Json(json!({
                "collections": ["sentinel-2-l2a"],
                "limit": 1,
                "token": "next:S2A_MSIL2A_20230216T150721_R082_T19PHS_20230217T082924"
            })))
            .with_body(include_str!("../mocks/page-2.json"))
            .with_header("content-type", "application/geo+json")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url()).unwrap();
        let search = Search::new().collection("sentinel-2-l2a").limit(1);
        let items: Vec<_> = client
            .search(search)
            .map(|result| result.unwrap())
            .take(2)
            .collect()
            .await;
        page_1.assert_async().await;
        page_2.assert_async().await;
        assert_eq!(items.len(), 2);
        assert!(items[0]["id"] != items[1]["id"]);
    }
}
