use crate::{Error, Page, Result};
use serde::de::DeserializeOwned;
use stac::{Collection, Item};
use stac_api::Search;
use tokio_postgres::{
    types::{ToSql, WasNull},
    GenericClient, Row,
};

/// A **pgstac** client.
///
/// Not every **pgstac** function is provided, and some names are changed to
/// match Rust conventions.
///
/// We don't own the inner client because we want to be able to work with
/// references, e.g. those returned by
/// [bb8_postgres](https://github.com/djc/bb8).
#[derive(Debug)]
pub struct Client<'a, C>(&'a C)
where
    C: GenericClient;

impl<'a, C: GenericClient> Client<'a, C> {
    /// Creates a new client.
    ///
    /// # Examples
    ///
    /// ```
    /// use pgstac::Client;
    /// use tokio_postgres::NoTls;
    ///
    /// let config = "postgresql://username:password@localhost:5432/postgis";
    /// # tokio_test::block_on(async {
    /// let (mut client, connection) = tokio_postgres::connect(config, NoTls).await.unwrap();
    /// let client = Client::new(&client);
    /// # });
    /// ```
    pub fn new(client: &C) -> Client<C> {
        Client(client)
    }

    /// Returns the **pgstac** version.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pgstac::Client;
    /// use tokio_postgres::NoTls;
    /// let config = "postgresql://username:password@localhost:5432/postgis";
    /// # tokio_test::block_on(async {
    /// let (mut client, connection) = tokio_postgres::connect(config, NoTls).await.unwrap();
    /// let client = Client::new(&client);
    /// let version = client.version().await.unwrap();
    /// # });
    /// ```
    pub async fn version(&self) -> Result<String> {
        self.string("get_version", &[]).await
    }

    /// Returns the value of the `context` **pgstac** setting.
    ///
    /// This setting defaults to "off".  See [the **pgstac**
    /// docs](https://github.com/stac-utils/pgstac/blob/main/docs/src/pgstac.md#pgstac-settings)
    /// for more information on the settings and their meaning.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pgstac::Client;
    /// use tokio_postgres::NoTls;
    /// let config = "postgresql://username:password@localhost:5432/postgis";
    /// # tokio_test::block_on(async {
    /// let (mut client, connection) = tokio_postgres::connect(config, NoTls).await.unwrap();
    /// let client = Client::new(&client);
    /// assert!(!client.context().await.unwrap());
    /// # });
    /// ```
    pub async fn context(&self) -> Result<bool> {
        self.string("get_setting", &[&"context"])
            .await
            .map(|value| value == "on")
    }

    /// Sets the value of the `context` **pgstac** setting.
    ///
    /// This setting defaults to "off".  See [the **pgstac**
    /// docs](https://github.com/stac-utils/pgstac/blob/main/docs/src/pgstac.md#pgstac-settings)
    /// for more information on the settings and their meaning.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use pgstac::Client;
    /// use tokio_postgres::NoTls;
    /// let config = "postgresql://username:password@localhost:5432/postgis";
    /// # tokio_test::block_on(async {
    /// let (mut client, connection) = tokio_postgres::connect(config, NoTls).await.unwrap();
    /// let client = Client::new(&client);
    /// client.set_context(true).await.unwrap();
    /// # });
    /// ```
    pub async fn set_context(&self, enable: bool) -> Result<()> {
        let value = if enable { "on" } else { "off" };
        self.0.execute(
            "INSERT INTO pgstac_settings (name, value) VALUES ('context', $1) ON CONFLICT ON CONSTRAINT pgstac_settings_pkey DO UPDATE SET value = excluded.value;",
            &[&value],
        ).await.map(|_| ()).map_err(Error::from)
    }

    /// Fetches all collections.
    pub async fn collections(&self) -> Result<Vec<Collection>> {
        self.vec("all_collections", &[]).await
    }

    /// Fetches a collection by id.
    pub async fn collection(&self, id: &str) -> Result<Option<Collection>> {
        self.opt("get_collection", &[&id]).await
    }

    /// Adds a collection.
    pub async fn add_collection(&self, collection: Collection) -> Result<()> {
        let collection = serde_json::to_value(collection)?;
        self.void("create_collection", &[&collection]).await
    }

    /// Adds or updates a collection.
    pub async fn upsert_collection(&self, collection: Collection) -> Result<()> {
        let collection = serde_json::to_value(collection)?;
        self.void("upsert_collection", &[&collection]).await
    }

    /// Updates a collection.
    pub async fn update_collection(&self, collection: Collection) -> Result<()> {
        let collection = serde_json::to_value(collection)?;
        self.void("update_collection", &[&collection]).await
    }

    /// Deletes a collection.
    pub async fn delete_collection(&self, id: &str) -> Result<()> {
        self.void("delete_collection", &[&id]).await
    }

    /// Fetches an item.
    pub async fn item(&self, id: &str, collection: &str) -> Result<Option<Item>> {
        self.opt("get_item", &[&id, &collection]).await
    }

    /// Adds an item.
    pub async fn add_item(&self, item: Item) -> Result<()> {
        let item = serde_json::to_value(item)?;
        self.void("create_item", &[&item]).await
    }

    /// Adds items.
    pub async fn add_items(&self, items: &[Item]) -> Result<()> {
        let items = serde_json::to_value(items)?;
        self.void("create_items", &[&items]).await
    }

    /// Updates an item.
    pub async fn update_item(&self, item: Item) -> Result<()> {
        let item = serde_json::to_value(item)?;
        self.void("update_item", &[&item]).await
    }

    /// Upserts an item.
    pub async fn upsert_item(&self, item: Item) -> Result<()> {
        let item = serde_json::to_value(item)?;
        self.void("upsert_item", &[&item]).await
    }

    /// Upserts items.
    pub async fn upsert_items(&self, items: &[Item]) -> Result<()> {
        let items = serde_json::to_value(items)?;
        self.void("upsert_items", &[&items]).await
    }

    /// Searches for items.
    pub async fn search(&self, search: Search) -> Result<Page> {
        let search = serde_json::to_value(search)?;
        self.value("search", &[&search]).await
    }

    async fn query_one(
        &self,
        function: &str,
        params: &[&(dyn ToSql + Sync)],
    ) -> std::result::Result<Row, tokio_postgres::Error> {
        let param_string = (0..params.len())
            .map(|i| format!("${}", i + 1))
            .collect::<Vec<_>>()
            .join(", ");
        let query = format!("SELECT * from pgstac.{}({})", function, param_string);
        self.0.query_one(&query, params).await
    }

    async fn string(&self, function: &str, params: &[&(dyn ToSql + Sync)]) -> Result<String> {
        let row = self.query_one(function, params).await?;
        row.try_get(function).map_err(Error::from)
    }

    async fn vec<T>(&self, function: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Vec<T>>
    where
        T: DeserializeOwned,
    {
        if let Some(value) = self.opt(function, params).await? {
            Ok(value)
        } else {
            Ok(Vec::new())
        }
    }

    async fn opt<T>(&self, function: &str, params: &[&(dyn ToSql + Sync)]) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        match self.value(function, params).await {
            Ok(value) => Ok(value),
            Err(err) => match err {
                Error::TokioPostgres(err) => {
                    if let Some(err) = err.into_source() {
                        if err.downcast_ref::<WasNull>().is_some() {
                            Ok(None)
                        } else {
                            Err(Error::from(err))
                        }
                    } else {
                        Err(Error::Unknown)
                    }
                }
                _ => Err(err),
            },
        }
    }

    async fn value<T>(&self, function: &str, params: &[&(dyn ToSql + Sync)]) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let row = self.query_one(function, params).await?;
        let value = row.try_get(function)?;
        serde_json::from_value(value).map_err(Error::from)
    }

    async fn void(&self, function: &str, params: &[&(dyn ToSql + Sync)]) -> Result<()> {
        let _ = self.query_one(function, params).await?;
        Ok(())
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::Client;
    use geojson::{Geometry, Value};
    use pgstac_test::pgstac_test;
    use serde_json::{json, Map};
    use stac::{Collection, Item};
    use stac_api::{Fields, Filter, Search, Sortby};
    use std::sync::Mutex;
    use tokio_postgres::Transaction;

    // This is an absolutely heinous way to ensure that only one test is hitting
    // the DB at a time -- the MUTEX is used in the pgstac-test crate as part of
    // the code generated by `pgstac_test`.
    //
    // There's got to be a better way.
    pub(crate) static MUTEX: Mutex<()> = Mutex::new(());

    fn longmont() -> Geometry {
        Geometry::new(Value::Point(vec![-105.1019, 40.1672]))
    }

    #[pgstac_test]
    async fn version(client: &Client<'_, Transaction<'_>>) {
        let _ = client.version().await.unwrap();
    }

    #[pgstac_test]
    async fn context(client: &Client<'_, Transaction<'_>>) {
        assert!(!client.context().await.unwrap());
    }

    #[pgstac_test]
    async fn set_context(client: &Client<'_, Transaction<'_>>) {
        client.set_context(true).await.unwrap();
        assert!(client.context().await.unwrap());
    }

    #[pgstac_test]
    async fn collections(client: &Client<'_, Transaction<'_>>) {
        assert!(client.collections().await.unwrap().is_empty());
        client
            .add_collection(Collection::new("an-id", "a description"))
            .await
            .unwrap();
        assert_eq!(client.collections().await.unwrap().len(), 1);
    }

    #[pgstac_test]
    async fn add_collection_duplicate(client: &Client<'_, Transaction<'_>>) {
        assert!(client.collections().await.unwrap().is_empty());
        let collection = Collection::new("an-id", "a description");
        client.add_collection(collection.clone()).await.unwrap();
        assert!(client.add_collection(collection).await.is_err());
    }

    #[pgstac_test]
    async fn upsert_collection(client: &Client<'_, Transaction<'_>>) {
        assert!(client.collections().await.unwrap().is_empty());
        let mut collection = Collection::new("an-id", "a description");
        client.upsert_collection(collection.clone()).await.unwrap();
        collection.title = Some("a title".to_string());
        client.upsert_collection(collection).await.unwrap();
        assert_eq!(
            client
                .collection("an-id")
                .await
                .unwrap()
                .unwrap()
                .title
                .unwrap(),
            "a title"
        );
    }

    #[pgstac_test]
    async fn update_collection(client: &Client<'_, Transaction<'_>>) {
        let mut collection = Collection::new("an-id", "a description");
        client.add_collection(collection.clone()).await.unwrap();
        assert!(client
            .collection("an-id")
            .await
            .unwrap()
            .unwrap()
            .title
            .is_none());
        collection.title = Some("a title".to_string());
        client.update_collection(collection).await.unwrap();
        assert_eq!(client.collections().await.unwrap().len(), 1);
        assert_eq!(
            client
                .collection("an-id")
                .await
                .unwrap()
                .unwrap()
                .title
                .unwrap(),
            "a title"
        );
    }

    #[pgstac_test]
    async fn update_collection_does_not_exit(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("an-id", "a description");
        assert!(client.update_collection(collection).await.is_err());
    }

    #[pgstac_test]
    async fn collection_not_found(client: &Client<'_, Transaction<'_>>) {
        assert!(client.collection("not-an-id").await.unwrap().is_none());
    }

    #[pgstac_test]
    async fn delete_collection(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("an-id", "a description");
        client.add_collection(collection.clone()).await.unwrap();
        assert!(client.collection("an-id").await.unwrap().is_some());
        client.delete_collection("an-id").await.unwrap();
        assert!(client.collection("an-id").await.unwrap().is_none());
    }

    #[pgstac_test]
    async fn delete_collection_does_not_exist(client: &Client<'_, Transaction<'_>>) {
        assert!(client.delete_collection("not-an-id").await.is_err());
    }

    #[pgstac_test]
    async fn item(client: &Client<'_, Transaction<'_>>) {
        assert!(client
            .item("an-id", "collection-id")
            .await
            .unwrap()
            .is_none());
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        assert_eq!(
            client
                .item("an-id", "collection-id")
                .await
                .unwrap()
                .unwrap(),
            item
        );
    }

    #[pgstac_test]
    async fn item_without_collection(client: &Client<'_, Transaction<'_>>) {
        let item = Item::new("an-id");
        assert!(client.add_item(item.clone()).await.is_err());
    }

    #[pgstac_test]
    async fn update_item(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        item.properties
            .additional_fields
            .insert("foo".into(), "bar".into());
        client.update_item(item).await.unwrap();
        assert_eq!(
            client
                .item("an-id", "collection-id")
                .await
                .unwrap()
                .unwrap()
                .properties
                .additional_fields["foo"],
            "bar"
        );
    }

    #[pgstac_test]
    async fn upsert_item(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.upsert_item(item.clone()).await.unwrap();
        client.upsert_item(item).await.unwrap();
    }

    #[pgstac_test]
    async fn add_items(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        let mut other_item = item.clone();
        other_item.id = "other-id".to_string();
        client.add_items(&[item, other_item]).await.unwrap();
        assert!(client
            .item("an-id", "collection-id")
            .await
            .unwrap()
            .is_some());
        assert!(client
            .item("other-id", "collection-id")
            .await
            .unwrap()
            .is_some());
    }

    #[pgstac_test]
    async fn upsert_items(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        let mut other_item = item.clone();
        other_item.id = "other-id".to_string();
        let items = vec![item, other_item];
        client.upsert_items(&items).await.unwrap();
        client.upsert_items(&items).await.unwrap();
    }

    #[pgstac_test]
    async fn search_everything(client: &Client<'_, Transaction<'_>>) {
        assert!(client
            .search(Search::default())
            .await
            .unwrap()
            .features
            .is_empty());
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        assert_eq!(
            client.search(Search::default()).await.unwrap().features[0],
            *serde_json::to_value(item).unwrap().as_object().unwrap()
        );
    }

    #[pgstac_test]
    async fn search_ids(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        let search = Search {
            ids: Some(vec!["an-id".to_string()]),
            ..Default::default()
        };
        assert_eq!(client.search(search).await.unwrap().features.len(), 1);
        let search = Search {
            ids: Some(vec!["not-an-id".to_string()]),
            ..Default::default()
        };
        assert!(client.search(search).await.unwrap().features.is_empty());
    }

    #[pgstac_test]
    async fn search_collections(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        let search = Search {
            collections: Some(vec!["collection-id".to_string()]),
            ..Default::default()
        };
        assert_eq!(client.search(search).await.unwrap().features.len(), 1);
        let search = Search {
            collections: Some(vec!["not-an-id".to_string()]),
            ..Default::default()
        };
        assert!(client.search(search).await.unwrap().features.is_empty());
    }

    #[pgstac_test]
    async fn search_limit(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        item.id = "another-id".to_string();
        client.add_item(item).await.unwrap();
        let mut search = Search::default();
        search.items.limit = Some(1);
        let page = client.search(search).await.unwrap();
        assert_eq!(page.features.len(), 1);
        assert_eq!(page.context.limit.unwrap(), 1);
    }

    #[pgstac_test]
    async fn search_bbox(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        let mut search = Search::default();
        search.items.bbox = Some(vec![-106., 40., -105., 41.].try_into().unwrap());
        assert_eq!(
            client.search(search.clone()).await.unwrap().features.len(),
            1
        );
        search.items.bbox = Some(vec![-106., 41., -105., 42.].try_into().unwrap());
        assert!(client.search(search).await.unwrap().features.is_empty());
    }

    #[pgstac_test]
    async fn search_datetime(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        item.properties.datetime = Some("2023-01-07T00:00:00Z".parse().unwrap());
        client.add_item(item.clone()).await.unwrap();
        let mut search = Search::default();
        search.items.datetime = Some("2023-01-07T00:00:00Z".to_string());
        assert_eq!(
            client.search(search.clone()).await.unwrap().features.len(),
            1
        );
        search.items.datetime = Some("2023-01-08T00:00:00Z".to_string());
        assert!(client.search(search).await.unwrap().features.is_empty());
    }

    #[pgstac_test]
    async fn search_intersects(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        let search = Search {
            intersects: Some(
                serde_json::from_value(
                    serde_json::to_value(Geometry::new(Value::Polygon(vec![vec![
                        vec![-106., 40.],
                        vec![-106., 41.],
                        vec![-105., 41.],
                        vec![-105., 40.],
                        vec![-106., 40.],
                    ]])))
                    .unwrap(),
                )
                .unwrap(),
            ),
            ..Default::default()
        };
        assert_eq!(client.search(search).await.unwrap().features.len(), 1);
        let search = Search {
            intersects: Some(
                serde_json::from_value(
                    serde_json::to_value(Geometry::new(Value::Polygon(vec![vec![
                        vec![-104., 40.],
                        vec![-104., 41.],
                        vec![-103., 41.],
                        vec![-103., 40.],
                        vec![-104., 40.],
                    ]])))
                    .unwrap(),
                )
                .unwrap(),
            ),
            ..Default::default()
        };
        assert!(client.search(search).await.unwrap().features.is_empty());
    }

    #[pgstac_test]
    async fn pagination(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.properties.datetime = Some("2023-01-08T00:00:00Z".parse().unwrap());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        item.id = "another-id".to_string();
        item.properties.datetime = Some("2023-01-07T00:00:00Z".parse().unwrap());
        client.add_item(item).await.unwrap();
        let mut search = Search::default();
        search.items.limit = Some(1);
        let page = client.search(search.clone()).await.unwrap();
        assert_eq!(page.features[0]["id"], "an-id");
        search
            .additional_fields
            .insert("token".to_string(), page.next_token().into());
        let page = client.search(search.clone()).await.unwrap();
        assert_eq!(page.features[0]["id"], "another-id");
        search
            .additional_fields
            .insert("token".to_string(), page.prev_token().into());
        let page = client.search(search).await.unwrap();
        assert_eq!(page.features[0]["id"], "an-id");
    }

    #[pgstac_test]
    async fn fields(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("an-id");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        item.properties
            .additional_fields
            .insert("foo".into(), 42.into());
        item.properties
            .additional_fields
            .insert("bar".into(), 43.into());
        client.add_item(item).await.unwrap();
        let mut search = Search::default();
        search.items.fields = Some(Fields {
            include: vec!["properties.foo".to_string()],
            exclude: vec!["properties.bar".to_string()],
        });
        let page = client.search(search).await.unwrap();
        let item = &page.features[0];
        assert!(item["properties"].as_object().unwrap().get("foo").is_some());
        assert!(item["properties"].as_object().unwrap().get("bar").is_none());
    }

    #[pgstac_test]
    async fn sortby(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("a");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        client.add_item(item.clone()).await.unwrap();
        item.id = "b".to_string();
        client.add_item(item).await.unwrap();
        let mut search = Search::default();
        search.items.sortby = Some(vec![Sortby::asc("id")]);
        let page = client.search(search.clone()).await.unwrap();
        assert_eq!(page.features[0]["id"], "a");
        assert_eq!(page.features[1]["id"], "b");

        search.items.sortby = Some(vec![Sortby::desc("id")]);
        let page = client.search(search).await.unwrap();
        assert_eq!(page.features[0]["id"], "b");
        assert_eq!(page.features[1]["id"], "a");
    }

    #[pgstac_test]
    async fn filter(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("a");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        item.properties
            .additional_fields
            .insert("foo".into(), 42.into());
        client.add_item(item.clone()).await.unwrap();
        item.id = "b".to_string();
        item.properties
            .additional_fields
            .insert("foo".into(), 43.into());
        client.add_item(item).await.unwrap();
        let mut filter = Map::new();
        filter.insert("op".into(), "=".into());
        filter.insert("args".into(), json!([{"property": "foo"}, 42]));
        let mut search = Search::default();
        search.items.filter = Some(Filter::Cql2Json(filter));
        let page = client.search(search).await.unwrap();
        assert_eq!(page.features.len(), 1);
    }

    #[pgstac_test]
    async fn query(client: &Client<'_, Transaction<'_>>) {
        let collection = Collection::new("collection-id", "a description");
        client.add_collection(collection).await.unwrap();
        let mut item = Item::new("a");
        item.collection = Some("collection-id".to_string());
        item.geometry = Some(longmont());
        item.properties
            .additional_fields
            .insert("foo".into(), 42.into());
        client.add_item(item.clone()).await.unwrap();
        item.id = "b".to_string();
        item.properties
            .additional_fields
            .insert("foo".into(), 43.into());
        client.add_item(item).await.unwrap();
        let mut query = Map::new();
        query.insert("foo".into(), json!({"eq": 42}));
        let mut search = Search::default();
        search.items.query = Some(query);
        let page = client.search(search).await.unwrap();
        assert_eq!(page.features.len(), 1);
    }
}
