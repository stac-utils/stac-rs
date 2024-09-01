use crate::{Client, Error, Result};
use async_stream::try_stream;
use futures_core::stream::Stream;
use futures_util::{pin_mut, StreamExt};
use reqwest::Method;
use stac::{Collection, Links};
use stac_api::{GetItems, Item, ItemCollection, Items, Search, UrlBuilder};
use tokio::{
    sync::mpsc::{self, error::SendError},
    task::JoinHandle,
};

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
        ApiClient::with_client(Client::new(), url)
    }

    /// Creates a new API client with the given [Client].
    ///
    /// Useful if you want to customize the behavior of the underlying `Client`,
    /// as documented in [Client::new].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_async::{Client, ApiClient};
    /// let client = Client::new();
    /// let api_client = ApiClient::with_client(client, "https://earth-search.aws.element84.com/v1/").unwrap();
    /// ```
    pub fn with_client(client: Client, url: &str) -> Result<ApiClient> {
        Ok(ApiClient {
            client,
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

    /// Returns a stream of items belonging to a collection, using the [items
    /// endpoint](https://github.com/radiantearth/stac-api-spec/tree/main/ogcapi-features#collection-items-collectionscollectioniditems).
    ///
    /// The `items` argument can be used to filter, sort, and otherwise
    /// configure the request.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac_api::Items;
    /// use stac_async::ApiClient;
    /// use futures_util::stream::StreamExt;
    ///
    /// let client = ApiClient::new("https://planetarycomputer.microsoft.com/api/stac/v1").unwrap();
    /// let items = Items {
    ///     limit: Some(1),
    ///     ..Default::default()
    /// };
    /// # tokio_test::block_on(async {
    /// let items: Vec<_> = client
    ///     .items("sentinel-2-l2a", items)
    ///     .await
    ///     .unwrap()
    ///     .map(|result| result.unwrap())
    ///     .collect()
    ///     .await;
    /// assert_eq!(items.len(), 1);
    /// # })
    /// ```
    pub async fn items(
        &self,
        id: &str,
        items: impl Into<Option<Items>>,
    ) -> Result<impl Stream<Item = Result<Item>>> {
        let url = self.url_builder.items(id)?; // TODO HATEOS
        let items = if let Some(items) = items.into() {
            Some(GetItems::try_from(items)?)
        } else {
            None
        };
        let page: Option<ItemCollection> = self
            .client
            .request(Method::GET, url.clone(), items.as_ref(), None)
            .await?;
        if let Some(page) = page {
            Ok(stream_items(self.client.clone(), page, self.channel_buffer))
        } else {
            Err(Error::NotFound(url))
        }
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
    /// let mut search = Search { collections: Some(vec!["sentinel-2-l2a".to_string()]), ..Default::default() };
    /// search.items.limit = Some(1);
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
    let handle: JoinHandle<std::result::Result<(), SendError<_>>> = tokio::spawn(async move {
        let pages = stream_pages(client, page);
        pin_mut!(pages);
        while let Some(result) = pages.next().await {
            match result {
                Ok(page) => tx.send(Ok(page)).await?,
                Err(err) => {
                    tx.send(Err(err)).await?;
                    return Ok(());
                }
            }
        }
        Ok(())
    });
    try_stream! {
        while let Some(result) = rx.recv().await {
            let page = result?;
            for item in page.items {
                yield item;
            }
        }
        let _ = handle.await?;
    }
}

fn stream_pages(
    client: Client,
    mut page: ItemCollection,
) -> impl Stream<Item = Result<ItemCollection>> {
    try_stream! {
        loop {
            if page.items.is_empty() {
                break;
            }
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
    use stac_api::{ItemCollection, Items, Search};
    use url::Url;

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
            serde_json::from_str(include_str!("../mocks/search-page-1.json")).unwrap();
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
            .with_body(include_str!("../mocks/search-page-2.json"))
            .with_header("content-type", "application/geo+json")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url()).unwrap();
        let mut search = Search {
            collections: Some(vec!["sentinel-2-l2a".to_string()]),
            ..Default::default()
        };
        search.items.limit = Some(1);
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

    #[tokio::test]
    async fn items_with_paging() {
        let mut server = Server::new_async().await;
        let mut page_1_body: ItemCollection =
            serde_json::from_str(include_str!("../mocks/items-page-1.json")).unwrap();
        let mut next_link = page_1_body.link("next").unwrap().clone();
        let url: Url = next_link.href.parse().unwrap();
        let query = url.query().unwrap();
        next_link.href = format!(
            "{}/collections/sentinel-2-l2a/items?{}",
            server.url(),
            query
        );
        page_1_body.set_link(next_link);
        let page_1 = server
            .mock("GET", "/collections/sentinel-2-l2a/items?limit=1")
            .with_body(serde_json::to_string(&page_1_body).unwrap())
            .with_header("content-type", "application/geo+json")
            .create_async()
            .await;
        let page_2 = server
            .mock("GET", "/collections/sentinel-2-l2a/items?limit=1&token=next:S2A_MSIL2A_20230216T235751_R087_T52CEB_20230217T134604")
            .with_body(include_str!("../mocks/items-page-2.json"))
            .with_header("content-type", "application/geo+json")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url()).unwrap();
        let items = Items {
            limit: Some(1),
            ..Default::default()
        };
        let items: Vec<_> = client
            .items("sentinel-2-l2a", Some(items))
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

    #[tokio::test]
    async fn stop_on_empty_page() {
        let mut server = Server::new_async().await;
        let mut page_body: ItemCollection =
            serde_json::from_str(include_str!("../mocks/items-page-1.json")).unwrap();
        let mut next_link = page_body.link("next").unwrap().clone();
        let url: Url = next_link.href.parse().unwrap();
        let query = url.query().unwrap();
        next_link.href = format!(
            "{}/collections/sentinel-2-l2a/items?{}",
            server.url(),
            query
        );
        page_body.set_link(next_link);
        page_body.items = vec![];
        let page = server
            .mock("GET", "/collections/sentinel-2-l2a/items?limit=1")
            .with_body(serde_json::to_string(&page_body).unwrap())
            .with_header("content-type", "application/geo+json")
            .create_async()
            .await;

        let client = ApiClient::new(&server.url()).unwrap();
        let items = Items {
            limit: Some(1),
            ..Default::default()
        };
        let items: Vec<_> = client
            .items("sentinel-2-l2a", Some(items))
            .await
            .unwrap()
            .map(|result| result.unwrap())
            .collect()
            .await;
        page.assert_async().await;
        assert!(items.is_empty());
    }
}
