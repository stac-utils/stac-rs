mod memory;
#[cfg(feature = "pgstac")]
mod pgstac;

use crate::{Error, Result};
use async_trait::async_trait;
pub use memory::MemoryBackend;
#[cfg(feature = "pgstac")]
pub use pgstac::PgstacBackend;
use stac::{Collection, Item, Value};
use stac_api::{ItemCollection, Items, Search};

/// Storage backend for a STAC API.
#[async_trait]
pub trait Backend: Clone + Sync + Send + 'static {
    /// Returns true if this backend has item search capabilities.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_server::{MemoryBackend, Backend};
    ///
    /// assert_eq!(MemoryBackend::new().has_item_search(), cfg!(feature = "memory-item-search"));
    /// ```
    fn has_item_search(&self) -> bool;

    /// Adds collections and items from hrefs.
    ///
    /// A default implementation is provided.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_server::{MemoryBackend, Backend};
    /// let mut backend = MemoryBackend::new();
    /// # tokio_test::block_on(async {
    /// backend.add_from_hrefs(&[
    ///     "tests/data/collection.json".to_string(),
    ///     "tests/data/feature.geojson".to_string(),
    /// ]);
    /// # })
    /// ```
    async fn add_from_hrefs(&mut self, hrefs: &[String]) -> Result<()> {
        let mut items = Vec::new();
        for href in hrefs {
            let value: Value = stac_async::read(href).await?;
            match value {
                Value::Item(item) => items.push(item),
                Value::Catalog(catalog) => {
                    return Err(Error::Backend(format!(
                        "cannot add catalog with id={} to the backend",
                        catalog.id
                    )))
                }
                Value::Collection(collection) => self.add_collection(collection).await?,
                Value::ItemCollection(item_collection) => {
                    items.extend(item_collection.items.into_iter())
                }
            }
        }
        for item in items {
            self.add_item(item).await?;
        }
        Ok(())
    }

    /// Returns all collections.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_server::{MemoryBackend, Backend};
    /// let backend = MemoryBackend::new();
    /// # tokio_test::block_on(async {
    /// let collections = backend.collections().await.unwrap();
    /// assert!(collections.is_empty());
    /// # })
    /// ```
    async fn collections(&self) -> Result<Vec<Collection>>;

    /// Returns a single collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_server::{MemoryBackend, Backend};
    /// let backend = MemoryBackend::new();
    /// # tokio_test::block_on(async {
    /// let collection = backend.collection("does-not-exist").await.unwrap();
    /// assert!(collection.is_none());
    /// # })
    /// ```
    async fn collection(&self, id: &str) -> Result<Option<Collection>>;

    /// Adds a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Collection;
    /// use stac_server::{MemoryBackend, Backend};
    ///
    /// let mut backend = MemoryBackend::new();
    /// # tokio_test::block_on(async {
    /// backend.add_collection(Collection::new("an-id", "a description")).await.unwrap();
    /// # })
    /// ```
    async fn add_collection(&mut self, collection: Collection) -> Result<()>;

    /// Adds an item.
    ///
    /// If the item doesn't have its `collection` field set, or a collection
    /// with that id does not exist in the backend, throws an error.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Collection, Item};
    /// use stac_server::{MemoryBackend, Backend};
    ///
    /// let mut backend = MemoryBackend::new();
    /// # tokio_test::block_on(async {
    /// assert!(backend.add_item(Item::new("item-id")).await.is_err());
    ///
    /// backend.add_collection(Collection::new("collection-id", "a description")).await.unwrap();
    /// backend.add_item(Item::new("item-id").collection("collection-id")).await.unwrap();
    /// # })
    /// ```
    async fn add_item(&mut self, item: Item) -> Result<()>;

    /// Retrieves items for a given collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Collection, Item};
    /// use stac_api::Items;
    /// use stac_server::{MemoryBackend, Backend};
    ///
    /// let mut backend = MemoryBackend::new();
    /// # tokio_test::block_on(async {
    /// backend.add_collection(Collection::new("collection-id", "a description")).await.unwrap();
    /// backend.add_item(Item::new("item-id").collection("collection-id")).await.unwrap();
    /// let items = backend.items("collection-id", Items::default()).await.unwrap();
    /// # })
    /// ```
    async fn items(&self, collection_id: &str, items: Items) -> Result<Option<ItemCollection>>;

    /// Retrieves an item from a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Collection, Item};
    /// use stac_server::{MemoryBackend, Backend};
    ///
    /// let mut backend = MemoryBackend::new();
    /// # tokio_test::block_on(async {
    /// backend.add_collection(Collection::new("collection-id", "a description")).await.unwrap();
    /// backend.add_item(Item::new("item-id").collection("collection-id")).await.unwrap();
    /// let item = backend.item("collection-id", "item-id").await.unwrap().unwrap();
    /// # })
    /// ```
    async fn item(&self, collection_id: &str, item_id: &str) -> Result<Option<Item>>;

    /// Searches a backend.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_api::Search;
    /// use stac_server::{MemoryBackend, Backend};
    ///
    /// let mut backend = MemoryBackend::new();
    /// # tokio_test::block_on(async {
    /// let item_collection = backend.search(Search::default()).await.unwrap();
    /// # })
    /// ```
    async fn search(&self, search: Search) -> Result<ItemCollection>;
}
