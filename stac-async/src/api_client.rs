use crate::{Client, Error, Result};
use async_stream::try_stream;
use futures_core::stream::Stream;
use futures_util::{pin_mut, StreamExt};
use stac::{Collection, Links};
use stac_api::{Item, ItemCollection, Search, UrlBuilder};
use tokio::sync::mpsc;

const DEFAULT_CHANNEL_BUFFER: usize = 4;

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
        // TODO support HATEOS (aka look up the urls from the root catalog)
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
    ///     .await
    ///     .unwrap()
    ///     .map(|result| result.unwrap())
    ///     .collect()
    ///     .await;
    /// assert_eq!(items.len(), 1);
    /// # })
    /// ```
    pub async fn search(&self, search: Search) -> Result<impl Stream<Item = Result<Item>>> {
        let url = self.url_builder.search().clone();
        // TODO support GET
        let page: Option<ItemCollection> = self.client.post(url.clone(), &search).await?;
        if let Some(page) = page {
            Ok(stream_items(self.client.clone(), page, self.channel_buffer))
        } else {
            Err(Error::NotFound(url))
        }
    }
}

fn stream_items(
    client: Client,
    page: ItemCollection,
    channel_buffer: usize,
) -> impl Stream<Item = Result<Item>> {
    let (tx, mut rx) = mpsc::channel(channel_buffer);
    tokio::spawn(async move {
        let pages = stream_pages(client, page);
        pin_mut!(pages);
        while let Some(result) = pages.next().await {
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

fn stream_pages(
    client: Client,
    mut page: ItemCollection,
) -> impl Stream<Item = Result<ItemCollection>> {
    try_stream! {
        loop {
            let next_link = page.link("next").cloned();
            yield page;
            if let Some(next_link) = next_link {
                if let Some(next_page) = client.request_from_link(next_link).await? {
                    page = next_page;
                } else {
                    break;
                }
            } else {
                break;
            }
        }
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
            .await
            .unwrap()
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
