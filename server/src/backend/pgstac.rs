use crate::{Backend, Error, Result};
use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use pgstac::Client;
use serde_json::Map;
use stac::{Collection, Item};
use stac_api::{ItemCollection, Items, Search};
use tokio_postgres::tls::NoTls;

/// A backend for a [pgstac](https://github.com/stac-utils/pgstac) database.
#[derive(Clone, Debug)]
pub struct PgstacBackend {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl PgstacBackend {
    /// Creates a new PgstacBackend from a string-like configuration.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use stac_server::PgstacBackend;
    /// # tokio_test::block_on(async {
    /// let backend = PgstacBackend::new_from_stringlike("postgresql://username:password@localhost:5432/postgis").await.unwrap();
    /// # })
    /// ```
    pub async fn new_from_stringlike(params: impl ToString) -> Result<PgstacBackend> {
        let connection_manager = PostgresConnectionManager::new_from_stringlike(params, NoTls)?;
        let pool = Pool::builder().build(connection_manager).await?;
        Ok(PgstacBackend::new(pool))
    }

    fn new(pool: Pool<PostgresConnectionManager<NoTls>>) -> PgstacBackend {
        PgstacBackend { pool }
    }
}

impl Backend for PgstacBackend {
    fn has_item_search(&self) -> bool {
        true
    }

    async fn add_collection(&mut self, collection: Collection) -> Result<()> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.add_collection(collection).await.map_err(Error::from)
    }

    async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.collection(id).await.map_err(Error::from)
    }

    async fn collections(&self) -> Result<Vec<Collection>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.collections().await.map_err(Error::from)
    }

    async fn add_item(&mut self, item: Item) -> Result<()> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client.add_item(item).await.map_err(Error::from)
    }

    async fn items(&self, collection_id: &str, items: Items) -> Result<Option<ItemCollection>> {
        // TODO should we check for collection existence?
        let search = items.search_collection(collection_id);
        self.search(search).await.map(Some)
    }

    async fn item(&self, collection_id: &str, item_id: &str) -> Result<Option<Item>> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        client
            .item(item_id, collection_id)
            .await
            .map_err(Error::from)
    }

    async fn search(&self, search: Search) -> Result<ItemCollection> {
        let client = self.pool.get().await?;
        let client = Client::new(&*client);
        let page = client.search(search).await?;
        let next_token = page.next_token();
        let prev_token = page.prev_token();
        let mut item_collection = ItemCollection::new(page.features)?;
        if let Some(next_token) = next_token {
            let mut next = Map::new();
            let _ = next.insert("token".into(), next_token.into());
            item_collection.next = Some(next);
        }
        if let Some(prev_token) = prev_token {
            let mut prev = Map::new();
            let _ = prev.insert("token".into(), prev_token.into());
            item_collection.prev = Some(prev);
        }
        item_collection.context = Some(page.context);
        Ok(item_collection)
    }
}
