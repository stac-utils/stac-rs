use crate::{Error, Result};
use stac_server::{Api, Backend, MemoryBackend};
use tokio::net::TcpListener;

/// Arguments for serving an API.
#[derive(Debug, clap::Args)]
pub struct Args {
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

impl crate::Args {
    pub async fn serve(&self, args: &Args) -> Result<()> {
        #[cfg(feature = "pgstac")]
        {
            if let Some(pgstac) = args.pgstac.as_deref() {
                let backend = stac_server::PgstacBackend::new_from_stringlike(pgstac).await?;
                self.load_and_serve(args, backend).await
            } else {
                let backend = MemoryBackend::new();
                self.load_and_serve(args, backend).await
            }
        }
        #[cfg(not(feature = "pgstac"))]
        {
            let backend = MemoryBackend::new();
            self.load_and_serve(args, backend).await
        }
    }

    async fn load_and_serve(&self, args: &Args, backend: impl Backend) -> Result<()> {
        tracing::warn!(
            "FIXME: once https://github.com/stac-utils/stac-rs/pull/513 lands, re-enable loading"
        );
        let root = format!("http://{}", &args.addr);
        let api = Api::new(backend, &root)?;
        let router = stac_server::routes::from_api(api);
        let listener = TcpListener::bind(&args.addr).await?;
        eprintln!("Serving a STAC API at {}", root);
        axum::serve(listener, router).await.map_err(Error::from)
    }
}
