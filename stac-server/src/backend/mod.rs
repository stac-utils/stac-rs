mod memory;
#[cfg(feature = "pgstac")]
mod pgstac;

use crate::Result;
use async_trait::async_trait;
pub use memory::MemoryBackend;
#[cfg(feature = "pgstac")]
pub use pgstac::PgstacBackend;
use stac::{Collection, Item};
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
    /// A default implementation is provided. If `auto_create_collections` is
    /// true, then, _if_ there is no collection for one or more items, a
    /// collection will be auto-created before adding the items. If
    /// `follow_links` is true, then `item` links on collections will be
    /// followed and added as well.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_server::{MemoryBackend, Backend};
    /// let mut backend = MemoryBackend::new();
    /// # tokio_test::block_on(async {
    /// backend.add_from_hrefs(vec![
    ///     "tests/data/collection.json".to_string(),
    ///     "tests/data/feature.geojson".to_string(),
    /// ], false, false);
    /// # })
    /// ```
    #[cfg(feature = "tokio")]
    async fn add_from_hrefs(
        &mut self,
        hrefs: Vec<String>,
        auto_create_collections: bool,
        follow_links: bool,
    ) -> Result<()> {
        use crate::Error;
        use stac::{Href, Links, Value};
        use std::collections::{HashMap, HashSet};
        use tokio::task::JoinSet;

        let mut set = JoinSet::new();
        for href in hrefs {
            let _ = set.spawn(async move { stac_async::read::<Value>(href).await });
        }

        let mut items: HashMap<Option<String>, Vec<Item>> = HashMap::new();
        let mut item_collection_ids = HashSet::new();
        let mut add_item = |mut item: Item| {
            item.remove_structural_links();
            if let Some(collection) = item.collection.as_ref() {
                let collection = collection.clone();
                let _ = item_collection_ids.insert(collection.clone());
                let _ = items.entry(Some(collection)).or_default().push(item);
            } else {
                let _ = items.entry(None).or_default().push(item);
            }
        };
        let mut item_set = JoinSet::new();
        let mut collection_ids = HashSet::new();
        while let Some(result) = set.join_next().await {
            let value = result??;
            match value {
                Value::Item(item) => add_item(item),
                Value::Catalog(catalog) => {
                    return Err(Error::Backend(format!(
                        "cannot add catalog with id={} to the backend",
                        catalog.id
                    )))
                }
                Value::Collection(mut collection) => {
                    if follow_links {
                        // TODO we could maybe merge this with `remove_structural_links`
                        let href = collection
                            .href()
                            .expect("we read it, so it should have an href")
                            .to_string();
                        collection.make_relative_links_absolute(href)?;
                        for link in collection.iter_item_links() {
                            let href = link.href.clone();
                            let _ =
                                item_set.spawn(async move { stac_async::read::<Item>(href).await });
                        }
                    }
                    collection.remove_structural_links();
                    let _ = collection_ids.insert(collection.id.clone());
                    self.add_collection(collection).await?
                }
                Value::ItemCollection(item_collection) => {
                    for item in item_collection.items {
                        add_item(item)
                    }
                }
            }
        }

        while let Some(result) = item_set.join_next().await {
            let item = result??;
            add_item(item);
        }

        if auto_create_collections {
            for id in item_collection_ids {
                if !collection_ids.contains(&id) {
                    let items = items
                        .get(&Some(id.clone())) // TODO can we get rid of this clone?
                        .expect("should have items for collection id");
                    let collection = Collection::from_id_and_items(id, items);
                    self.add_collection(collection).await?;
                }
            }
        }

        for (_, items) in items {
            for item in items {
                self.add_item(item).await?;
            }
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

#[cfg(test)]
mod tests {
    #[tokio::test]
    #[cfg(feature = "tokio")]
    async fn auto_create_collection() {
        use super::Backend;
        use crate::MemoryBackend;

        let mut backend = MemoryBackend::new();
        backend
            .add_from_hrefs(
                vec!["../spec-examples/v1.0.0/simple-item.json".to_string()],
                true,
                false,
            )
            .await
            .unwrap();
        let _ = backend
            .collection("simple-collection")
            .await
            .unwrap()
            .unwrap();
    }
}
