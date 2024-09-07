//! Use [object_store](https://docs.rs/object_store/latest/object_store/) to read and write STAC.

use crate::{Error, Result};
#[cfg(feature = "geoparquet")]
use geoarrow::io::parquet::GeoParquetWriterOptions;
use object_store::{path::Path, GetOptions, ObjectStore, PutOptions, PutResult};
use serde::{de::DeserializeOwned, Serialize};
use stac::{Href, Item, ItemCollection};
use std::future::Future;

/// Get STAC from an object store.
pub trait Get {
    /// Gets STAC from JSON in an object store.
    ///
    /// # Examples
    ///
    /// ```
    /// use object_store::{path::Path, local::LocalFileSystem};
    /// use stac::Item;
    /// use stac_async::object_store::Get;
    ///
    /// let store = LocalFileSystem::new();
    /// let location = Path::from_filesystem_path("examples/simple-item.json").unwrap();
    /// # tokio_test::block_on(async {
    /// let item: Item = store.get_json(&location).await.unwrap();
    /// # })
    /// ```
    fn get_json<T: Href + DeserializeOwned>(
        &self,
        location: &Path,
    ) -> impl Future<Output = Result<T>> {
        self.get_json_opts(location, GetOptions::default())
    }

    /// Gets STAC from JSON in an object store with options.
    fn get_json_opts<T: Href + DeserializeOwned>(
        &self,
        location: &Path,
        options: GetOptions,
    ) -> impl Future<Output = Result<T>>;

    /// Gets an [ItemCollection] from newline-delimited JSON in an object store.
    ///
    /// # Examples
    ///
    /// ```
    /// use object_store::{path::Path, local::LocalFileSystem};
    /// use stac::Item;
    /// use stac_async::object_store::Get;
    ///
    /// let store = LocalFileSystem::new();
    /// let location = Path::from_filesystem_path("data/items.ndjson").unwrap();
    /// # tokio_test::block_on(async {
    /// let items = store.get_ndjson(&location).await.unwrap();
    /// # })
    /// ```
    fn get_ndjson(&self, location: &Path) -> impl Future<Output = Result<ItemCollection>> {
        self.get_ndjson_opts(location, GetOptions::default())
    }

    /// Gets an [ItemCollection] from newline-delimited JSON in an object store with options.
    fn get_ndjson_opts(
        &self,
        location: &Path,
        options: GetOptions,
    ) -> impl Future<Output = Result<ItemCollection>>;

    /// Gets an [ItemCollection] from geoparquet in an object store.
    ///
    /// # Examples
    ///
    /// ```
    /// use object_store::{path::Path, local::LocalFileSystem};
    /// use stac::Item;
    /// use stac_async::object_store::Get;
    ///
    /// let store = LocalFileSystem::new();
    /// let location = Path::from_filesystem_path("data/extended-item.parquet").unwrap();
    /// # tokio_test::block_on(async {
    /// let items = store.get_geoparquet(&location).await.unwrap();
    /// # })
    /// ```
    #[cfg(feature = "geoparquet")]
    fn get_geoparquet(&self, location: &Path) -> impl Future<Output = Result<ItemCollection>> {
        self.get_geoparquet_opts(location, GetOptions::default())
    }

    /// Gets an [ItemCollection] from geoparquet in an object store with options.
    #[cfg(feature = "geoparquet")]
    fn get_geoparquet_opts(
        &self,
        location: &Path,
        options: GetOptions,
    ) -> impl Future<Output = Result<ItemCollection>>;
}

/// Puts STAC to an object store.
pub trait Put {
    /// Puts STAC to JSON in an object store.
    ///
    /// # Examples
    ///
    /// ```
    /// use object_store::{path::Path, memory::InMemory};
    /// use stac::Item;
    /// use stac_async::object_store::Put;
    ///
    /// let store = InMemory::new();
    /// let item: Item = stac::read("examples/simple-item.json").unwrap();
    /// let location = Path::from("simple-item.json");
    /// # tokio_test::block_on(async {
    /// let _ = store.put_json(&location, &item).await.unwrap();
    /// # })
    /// ```
    fn put_json<T: Serialize>(
        &self,
        location: &Path,
        value: &T,
    ) -> impl Future<Output = Result<PutResult>> {
        self.put_json_opts(location, value, PutOptions::default())
    }

    /// Puts STAC to JSON in an object store.
    fn put_json_opts<T: Serialize>(
        &self,
        location: &Path,
        value: &T,
        options: PutOptions,
    ) -> impl Future<Output = Result<PutResult>>;

    /// Puts an [ItemCollection] as newline-delimited JSON in an object store.
    ///
    /// # Examples
    ///
    /// ```
    /// use object_store::{path::Path, memory::InMemory};
    /// use stac::Item;
    /// use stac_async::object_store::Put;
    ///
    /// let store = InMemory::new();
    /// let item: Item = stac::read("examples/simple-item.json").unwrap();
    /// let location = Path::from("items.ndjson");
    /// # tokio_test::block_on(async {
    /// let _ = store.put_ndjson(&location, &vec![item].into()).await.unwrap();
    /// # })
    /// ```
    fn put_ndjson(
        &self,
        location: &Path,
        item_collection: &ItemCollection,
    ) -> impl Future<Output = Result<PutResult>> {
        self.put_ndjson_opts(location, item_collection, PutOptions::default())
    }

    /// Puts an [ItemCollection] as newline-delimited JSON in an object store.
    fn put_ndjson_opts(
        &self,
        location: &Path,
        item_collection: &ItemCollection,
        options: PutOptions,
    ) -> impl Future<Output = Result<PutResult>>;

    /// Puts an [ItemCollection] as geoparquet in an object store.
    ///
    /// # Examples
    ///
    /// ```
    /// use object_store::{path::Path, memory::InMemory};
    /// use stac::Item;
    /// use stac_async::object_store::Put;
    ///
    /// let store = InMemory::new();
    /// let item: Item = stac::read("examples/simple-item.json").unwrap();
    /// let location = Path::from("items.ndjson");
    /// # tokio_test::block_on(async {
    /// let _ = store.put_geoparquet(&location, vec![item].into()).await.unwrap();
    /// # })
    /// ```
    #[cfg(feature = "geoparquet")]
    fn put_geoparquet(
        &self,
        location: &Path,
        item_collection: ItemCollection,
    ) -> impl Future<Output = Result<PutResult>> {
        self.put_geoparquet_opts(
            location,
            item_collection,
            GeoParquetWriterOptions::default(),
            PutOptions::default(),
        )
    }

    /// Puts an [ItemCollection] as geoparquet in an object store with options.
    #[cfg(feature = "geoparquet")]
    fn put_geoparquet_opts(
        &self,
        location: &Path,
        item_collection: ItemCollection,
        geoparquet_writer_options: GeoParquetWriterOptions,
        put_options: PutOptions,
    ) -> impl Future<Output = Result<PutResult>>;
}

impl<O: ObjectStore> Get for O {
    async fn get_json_opts<T: Href + DeserializeOwned>(
        &self,
        location: &Path,
        options: GetOptions,
    ) -> Result<T> {
        let get_result = self.get_opts(location, options).await?;
        serde_json::from_slice(&get_result.bytes().await?).map_err(Error::from)
    }

    async fn get_ndjson_opts(
        &self,
        location: &Path,
        options: GetOptions,
    ) -> Result<ItemCollection> {
        let get_result = self.get_opts(location, options).await?;
        get_result
            .bytes()
            .await?
            .split(|c| *c == b'\n')
            .filter_map(|b| {
                if b.is_empty() {
                    None
                } else {
                    Some(serde_json::from_slice::<Item>(b).map_err(Error::from))
                }
            })
            .collect::<Result<Vec<Item>>>()
            .map(ItemCollection::from)
    }

    #[cfg(feature = "geoparquet")]
    async fn get_geoparquet_opts(
        &self,
        location: &Path,
        options: GetOptions,
    ) -> Result<ItemCollection> {
        let get_result = self.get_opts(location, options).await?;
        stac::geoparquet::from_reader(get_result.bytes().await?).map_err(Error::from)
    }
}

impl<O: ObjectStore> Put for O {
    async fn put_json_opts<T: Serialize>(
        &self,
        location: &Path,
        value: &T,
        options: PutOptions,
    ) -> Result<PutResult> {
        let mut buf = Vec::new();
        serde_json::to_writer(&mut buf, value)?;
        self.put_opts(location, buf.into(), options)
            .await
            .map_err(Error::from)
    }

    async fn put_ndjson_opts(
        &self,
        location: &Path,
        item_collection: &ItemCollection,
        options: PutOptions,
    ) -> Result<PutResult> {
        let mut buf = Vec::new();
        for item in &item_collection.items {
            serde_json::to_writer(&mut buf, item)?;
            buf.push(b'\n');
        }
        self.put_opts(location, buf.into(), options)
            .await
            .map_err(Error::from)
    }

    #[cfg(feature = "geoparquet")]
    async fn put_geoparquet_opts(
        &self,
        location: &Path,
        item_collection: ItemCollection,
        geoparquet_options: GeoParquetWriterOptions,
        put_options: PutOptions,
    ) -> Result<PutResult> {
        let mut buf = Vec::new();
        stac::geoparquet::to_writer_with_options(&mut buf, item_collection, &geoparquet_options)?;
        self.put_opts(location, buf.into(), put_options)
            .await
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    use super::{Get, Put};
    use object_store::{local::LocalFileSystem, memory::InMemory, path::Path};
    use stac::{Item, ItemCollection};

    #[tokio::test]
    async fn get_json() {
        let store = LocalFileSystem::new();
        let location = Path::from_filesystem_path("examples/simple-item.json").unwrap();
        let _: Item = store.get_json(&location).await.unwrap();
    }

    #[tokio::test]
    async fn put_json() {
        let store = InMemory::new();
        let item: Item = stac::read("examples/simple-item.json").unwrap();
        let location = Path::from("simple-item.json");
        let _ = store.put_json(&location, &item).await.unwrap();
        let item: Item = store.get_json(&location).await.unwrap();
        assert_eq!(item.id, "20201211_223832_CS2");
    }

    #[tokio::test]
    async fn get_ndjson() {
        let store = LocalFileSystem::new();
        let location = Path::from_filesystem_path("data/items.ndjson").unwrap();
        let item_collection = store.get_ndjson(&location).await.unwrap();
        assert_eq!(item_collection.len(), 2);
    }

    #[tokio::test]
    async fn put_ndjson() {
        let store = InMemory::new();
        let item: Item = stac::read("examples/simple-item.json").unwrap();
        let item_collection: ItemCollection = vec![item].into();
        let location = Path::from("items.json");
        let _ = store.put_ndjson(&location, &item_collection).await.unwrap();
        let items = store.get_ndjson(&location).await.unwrap();
        assert_eq!(items.len(), 1);
    }

    #[tokio::test]
    #[cfg(feature = "geoparquet")]
    async fn get_geoparquet() {
        let store = LocalFileSystem::new();
        let location = Path::from_filesystem_path("data/extended-item.parquet").unwrap();
        let item_collection = store.get_geoparquet(&location).await.unwrap();
        assert_eq!(item_collection.len(), 1);
    }

    #[tokio::test]
    #[cfg(feature = "geoparquet")]
    async fn put_geoparquet() {
        let store = InMemory::new();
        let item: Item = stac::read("examples/simple-item.json").unwrap();
        let item_collection: ItemCollection = vec![item].into();
        let location = Path::from("items.json");
        let _ = store
            .put_geoparquet(&location, item_collection)
            .await
            .unwrap();
        let items = store.get_geoparquet(&location).await.unwrap();
        assert_eq!(items.len(), 1);
    }
}
