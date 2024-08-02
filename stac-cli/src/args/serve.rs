use crate::{Printer, Result};
use clap::Args;
use stac_server::{Api, Backend, MemoryBackend};
use tokio::net::TcpListener;

/// Arguments for serving a STAC API.
#[derive(Args, Debug)]
pub struct ServeArgs {
    /// Hrefs of STAC collections and items to load before starting the server.
    href: Vec<String>,

    /// The pgstac connection string.
    #[arg(long)]
    pgstac: Option<String>,
}

impl ServeArgs {
    /// Serves a STAC API.
    #[allow(unused_variables)]
    pub async fn execute(&self, printer: Printer) -> Result<()> {
        let root = "http://127.0.0.1:7822";
        let addr = "127.0.0.1:7822";
        if let Some(pgstac) = self.pgstac.as_deref() {
            #[cfg(feature = "pgstac")]
            {
                let mut backend = stac_server::PgstacBackend::new_from_stringlike(pgstac).await?;
                if !self.href.is_empty() {
                    backend.add_from_hrefs(&self.href).await?;
                }
                let api = Api::new(backend, root)?;
                let router = stac_server::routes::from_api(api);
                let listener = TcpListener::bind(addr).await.unwrap();
                // TODO add "don't make me JSON" functionality to the printer
                println!("Serving a STAC API at {} using a pgstac backend", root);
                axum::serve(listener, router).await.unwrap();
            }
            #[cfg(not(feature = "pgstac"))]
            return Err(crate::Error::Custom(
                "stac-cli is not compiled with pgstac support".to_string(),
            ));
        } else {
            let mut backend = MemoryBackend::new();
            if !self.href.is_empty() {
                backend.add_from_hrefs(&self.href).await?;
            }
            let api = Api::new(backend, root)?;
            let router = stac_server::routes::from_api(api);
            let listener = TcpListener::bind(addr).await.unwrap();
            println!("Serving a STAC API at {} using a memory backend", root);
            axum::serve(listener, router).await.unwrap();
        };
        Ok(())
    }
}
