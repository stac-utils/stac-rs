use crate::Result;
use serde::Serialize;
use stac::{Assets, Href, Links};
use stac_async::Downloader;
use std::path::Path;

pub async fn download<A>(
    assets: A,
    directory: impl AsRef<Path>,
    create_directory: bool,
) -> Result<()>
where
    A: Assets + Href + Links + Serialize + Clone,
{
    Downloader::new(assets)?
        .create_directory(create_directory)
        .download(directory)
        .await?;
    Ok(())
}
