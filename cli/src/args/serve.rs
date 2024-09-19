use super::{Input, Run};
use crate::{Error, Result, Value};
use stac::{Collection, Item, Links};
use stac_server::{Api, Backend as _, MemoryBackend};
use std::collections::{HashMap, HashSet};
use tokio::{net::TcpListener, sync::mpsc::Sender, task::JoinSet};
use tracing::{info, warn};

const DEFAULT_COLLECTION_ID: &str = "auto-generated-collection";

/// Arguments for serving an API.
#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// Hrefs of collections, items, and item collections to load into the server on start
    ///
    /// If this is a single `-`, the data will be read from standard input.
    hrefs: Vec<String>,

    /// The address of the server.
    #[arg(short, long, default_value = "127.0.0.1:7822")]
    addr: String,

    /// The pgstac connection string, e.g. `postgresql://username:password@localhost:5432/postgis`
    ///
    /// If not provided an in-process memory backend will be used.
    #[arg(long)]
    #[cfg(feature = "pgstac")]
    pgstac: Option<String>,

    /// After loading a collection, load all of its item links
    #[arg(long, default_value_t = true)]
    load_collection_items: bool,

    /// Create collections for any items that don't have one
    #[arg(long, default_value_t = true)]
    create_collections: bool,
}

#[derive(Debug, Default)]
struct Info {
    backend: &'static str,
    collections: usize,
    items: usize,
}

impl Run for Args {
    async fn run(self, input: Input, _: Option<Sender<Value>>) -> Result<Option<Value>> {
        use stac::Value;

        let mut info = Info::default();
        let mut backend = {
            #[cfg(feature = "pgstac")]
            if let Some(pgstac) = self.pgstac {
                info!("creating a pgstac backend at {}", pgstac);
                info.backend = "pgstac";
                Backend::Pgstac(stac_server::PgstacBackend::new_from_stringlike(pgstac).await?)
            } else {
                info!("using a memory backend");
                info.backend = "memory";
                Backend::Memory(MemoryBackend::new())
            }
            #[cfg(not(feature = "pgstac"))]
            {
                info!("using a memory backend");
                info.backend = "memory";
                Backend::Memory(MemoryBackend::new())
            }
        };

        // TODO maybe this is worth pulling out into something more general?
        let mut join_set: JoinSet<Result<Value>> = JoinSet::new();
        let mut reading_from_stdin = false;
        for href in self.hrefs {
            if href == "-" {
                if reading_from_stdin {
                    continue;
                } else {
                    reading_from_stdin = true;
                }
            }
            let input = input.with_href(href);
            let _ = join_set.spawn(async move { input.get().await });
        }
        let mut item_join_set = JoinSet::new();
        let mut collections = HashSet::new();
        let mut items: HashMap<Option<String>, Vec<Item>> = HashMap::new();
        while let Some(result) = join_set.join_next().await {
            match result?? {
                Value::Catalog(catalog) => {
                    warn!(
                        "cannot load catalog with id '{}' into the server",
                        catalog.id
                    );
                }
                Value::Collection(mut collection) => {
                    if self.load_collection_items {
                        collection.make_relative_links_absolute()?;
                        for link in collection.iter_item_links() {
                            let href = link.href.to_string();
                            let input = input.with_href(href);
                            let _ = item_join_set.spawn(async move { input.get().await });
                        }
                    }
                    let _ = collections.insert(collection.id.clone());
                    backend.add_collection(collection).await?;
                    info.collections += 1;
                }
                Value::Item(item) => items.entry(item.collection.clone()).or_default().push(item),
                Value::ItemCollection(item_collection) => {
                    for item in item_collection {
                        items.entry(item.collection.clone()).or_default().push(item);
                    }
                }
            }
        }
        while let Some(result) = item_join_set.join_next().await {
            let value = result??;
            if let Value::Item(item) = value {
                items.entry(item.collection.clone()).or_default().push(item);
            } else {
                warn!("discarding non-item of type '{}'", value.type_name());
            }
        }
        if collections.contains(DEFAULT_COLLECTION_ID) && items.contains_key(&None) {
            warn!("a collection already exists with the default collection id '{}', discarding all items without a collection", DEFAULT_COLLECTION_ID);
            let _ = items.remove(&None);
        }
        for collection in collections {
            if let Some(items) = items.remove(&Some(collection)) {
                info.items += items.len();
                backend.add_items(items).await?;
            }
        }
        if self.create_collections {
            for (collection, items) in items
                .into_iter()
                .map(|(c, i)| (c.unwrap_or(DEFAULT_COLLECTION_ID.to_string()), i))
            {
                let collection = Collection::from_id_and_items(collection, &items);
                info.collections += 1;
                backend.add_collection(collection).await?;
                info.items += items.len();
                backend.add_items(items).await?;
            }
        } else if !items.is_empty() {
            let collection_ids: Vec<_> = items.into_keys().flatten().collect();
            warn!("--create-collections=false, but some items don't have collections and will not be loaded (collection ids: {})", collection_ids.join(","));
        }

        info!("starting server");
        match backend {
            Backend::Memory(backend) => start_server(backend, self.addr, info).await.and(Ok(None)),
            #[cfg(feature = "pgstac")]
            Backend::Pgstac(backend) => start_server(backend, self.addr, info).await.and(Ok(None)),
        }
    }
}

async fn start_server<B>(backend: B, addr: String, info: Info) -> Result<()>
where
    B: stac_server::Backend,
{
    let root = format!("http://{}", addr);
    let api = Api::new(backend, &root)?;
    let router = stac_server::routes::from_api(api);
    let listener = TcpListener::bind(addr).await?;
    eprintln!(
        "Serving a STAC API at {} using a {} backend{}",
        root,
        info.backend,
        info.counts()
    );
    axum::serve(listener, router).await.map_err(Error::from)
}

enum Backend {
    Memory(MemoryBackend),
    #[cfg(feature = "pgstac")]
    Pgstac(stac_server::PgstacBackend<pgstac::MakeRustlsConnect>),
}

impl Backend {
    async fn add_collection(&mut self, mut collection: Collection) -> Result<()> {
        info!("adding collection with id '{}'", collection.id);
        collection.remove_structural_links(); // TODO allow use to drain the child links first
        match self {
            Backend::Memory(backend) => backend
                .add_collection(collection)
                .await
                .map_err(Error::from),
            #[cfg(feature = "pgstac")]
            Backend::Pgstac(backend) => backend
                .add_collection(collection)
                .await
                .map_err(Error::from),
        }
    }

    async fn add_items(&mut self, items: Vec<Item>) -> Result<()> {
        assert!(!items.is_empty());
        info!(
            "adding {} items to collection '{}'",
            items.len(),
            items[0]
                .collection
                .as_ref()
                .expect("all items should have a collection"),
        );
        for mut item in items {
            item.remove_structural_links();
            match self {
                Backend::Memory(backend) => backend.add_item(item).await?,
                #[cfg(feature = "pgstac")]
                Backend::Pgstac(backend) => backend.add_item(item).await?,
            }
        }
        Ok(())
    }
}

impl Info {
    fn counts(&self) -> String {
        if self.collections > 0 || self.items > 0 {
            let collection = if self.collections == 1 {
                "collection"
            } else {
                "collections"
            };
            let item = if self.items == 1 { "item" } else { "items" };
            format!(
                " (loaded {} {} and {} {})",
                self.collections, collection, self.items, item
            )
        } else {
            String::new()
        }
    }
}
