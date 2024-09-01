use crate::{Output, Result, ServeArgs, Subcommand};
use stac_server::{Api, Backend, MemoryBackend};
use tokio::{net::TcpListener, sync::mpsc::Sender};

impl Subcommand {
    #[allow(unused_variables)]
    pub(crate) async fn serve(args: ServeArgs, sender: Sender<Output>) -> Result<()> {
        let root = "http://127.0.0.1:7822";
        let addr = "127.0.0.1:7822";
        if let Some(pgstac) = args.pgstac.as_deref() {
            #[cfg(feature = "pgstac")]
            {
                let mut backend = stac_server::PgstacBackend::new_from_stringlike(pgstac).await?;
                if !args.href.is_empty() {
                    backend
                        .add_from_hrefs(
                            args.href,
                            !args.dont_auto_create_collections,
                            !args.dont_follow_links,
                        )
                        .await?;
                }
                let api = Api::new(backend, root)?;
                let router = stac_server::routes::from_api(api);
                let listener = TcpListener::bind(addr).await.unwrap();
                sender
                    .send(format!("Serving a STAC API at {} using a pgstac backend", root).into())
                    .await?;
                axum::serve(listener, router).await.unwrap();
            }
            #[cfg(not(feature = "pgstac"))]
            return Err(crate::Error::Custom(
                "stac-cli is not compiled with pgstac support".to_string(),
            ));
        } else {
            let mut backend = MemoryBackend::new();
            if !args.href.is_empty() {
                backend
                    .add_from_hrefs(
                        args.href,
                        !args.dont_auto_create_collections,
                        !args.dont_follow_links,
                    )
                    .await?;
            }
            let api = Api::new(backend, root)?;
            let router = stac_server::routes::from_api(api);
            let listener = TcpListener::bind(addr).await.unwrap();
            sender
                .send(format!("Serving a STAC API at {} using a memory backend", root).into())
                .await?;
            axum::serve(listener, router).await.unwrap();
        };
        Ok(())
    }
}
