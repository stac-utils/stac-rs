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
    /// ```
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

    /// Searches an API.
    ///
    /// # Examples
    ///
    /// TODO
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
    use stac_api::Search;

    #[tokio::test]
    async fn collection_not_found() {
        let client = ApiClient::new("https://planetarycomputer.microsoft.com/api/stac/v1").unwrap();
        assert!(client
            .collection("not-a-collection")
            .await
            .unwrap()
            .is_none());
    }

    #[tokio::test]
    async fn search_with_paging() {
        let client = ApiClient::new("https://planetarycomputer.microsoft.com/api/stac/v1").unwrap();
        let search = Search::new().collection("sentinel-2-l2a").limit(1);
        let items: Vec<_> = client
            .search(search)
            .map(|result| result.unwrap())
            .take(2)
            .collect()
            .await;
        assert_eq!(items.len(), 2);
        assert!(items[0]["id"] != items[1]["id"]);
    }
}
