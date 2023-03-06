use crate::Result;
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use stac::{Asset, Assets, Collection, Href, Item, Link, Links, Value};
use std::path::Path;
use tokio::{fs::File, io::AsyncWriteExt, task::JoinSet};
use url::Url;

const DEFAULT_FILE_NAME: &str = "download.json";
const DEFAULT_WRITE_STAC: bool = true;
const DEFAULT_CREATE_DIRECTORY: bool = true;

/// Downloads all assets from a [Item](stac::Item) or [Collection](stac::Collection).
///
/// The STAC object's self href and asset hrefs are updated to point to the
/// downloaded locations. The original object's locations is included in a
/// "canonical" link.
///
/// # Examples
///
/// ```no_run
/// # tokio_test::block_on(async {
/// let value = stac_async::download("data/simple-item.json", "outdir").await.unwrap();
/// # })
/// ```
pub async fn download(href: impl ToString, directory: impl AsRef<Path>) -> Result<Value> {
    match crate::read(href).await? {
        Value::Item(item) => item.download(directory).await.map(|item| Value::Item(item)),
        Value::Collection(collection) => collection
            .download(directory)
            .await
            .map(|collection| Value::Collection(collection)),
        _ => unimplemented!(),
    }
}

/// Download the assets from anything that implements [Assets].
#[async_trait(?Send)]
pub trait Download: Assets + Links + Href + Serialize + Clone {
    /// Download the assets, and the object itself, to a directory on the local filesystem.
    ///
    /// # Examples
    ///
    /// [Item] implements [Download]:
    ///
    /// ```no_run
    /// use stac::{Item, Links};
    /// use stac_async::Download;
    ///
    /// let item: Item = stac::read("data/simple-item.json").unwrap();
    /// # tokio_test::block_on(async {
    /// let downloaded_item = item.download("outdir").await.unwrap();
    /// # })
    /// ```
    async fn download(self, directory: impl AsRef<Path>) -> Result<Self>
    where
        Self: Sized,
    {
        Downloader::new(self)?.download(directory).await
    }
}

/// A customizable download structure.
#[derive(Debug)]
pub struct Downloader<T: Links + Assets + Href + Serialize + Clone> {
    stac: T,
    client: Client,
    file_name: String,
    create_directory: bool,
    write_stac: bool,
}

#[derive(Debug)]
struct AssetDownloader {
    key: String,
    asset: Asset,
    client: Client,
}

impl<T: Links + Assets + Href + Serialize + Clone> Downloader<T> {
    /// Creates a new downloader.
    ///
    /// # Examples
    ///
    /// ```
    /// let item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// let downloader = stac_async::Downloader::new(item);
    /// ```
    pub fn new(mut stac: T) -> Result<Downloader<T>> {
        let file_name = if let Some(href) = stac.href().map(|href| href.to_string()) {
            stac.make_relative_links_absolute(&href)?;
            // TODO detect if this should be geojson or json
            stac.links_mut().push(Link::new(&href, "canonical"));
            href.rsplit_once('/')
                .map(|(_, file_name)| file_name.to_string())
        } else {
            let _ = stac.remove_relative_links();
            None
        };
        Ok(Downloader {
            stac,
            client: Client::new(),
            file_name: file_name.unwrap_or_else(|| DEFAULT_FILE_NAME.to_string()),
            create_directory: DEFAULT_CREATE_DIRECTORY,
            write_stac: DEFAULT_WRITE_STAC,
        })
    }

    /// Should the downloader create the output directory?
    ///
    /// Defaults to `true`.
    ///
    /// # Examples
    ///
    /// ```
    /// let item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// let downloader = stac_async::Downloader::new(item)
    ///     .unwrap()
    ///     .create_directory(false);
    /// ```
    pub fn create_directory(mut self, create_directory: bool) -> Downloader<T> {
        self.create_directory = create_directory;
        self
    }

    /// Downloads assets to the specified directory.
    ///
    /// Consumes this downloader, and returns the modified object.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// let item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// let downloader = stac_async::Downloader::new(item).unwrap();
    /// # tokio_test::block_on(async {
    /// let item = downloader.download("outdir").await.unwrap();
    /// # })
    /// ```
    pub async fn download(mut self, directory: impl AsRef<Path>) -> Result<T> {
        let mut join_set = JoinSet::new();
        let directory = directory.as_ref();
        if self.create_directory {
            tokio::fs::create_dir_all(directory).await?;
        }
        for asset_downloader in self.asset_downloaders() {
            let directory = directory.to_path_buf();
            let _ = join_set.spawn(async move { asset_downloader.download_to(directory) });
        }
        let path = directory.join(self.file_name);
        self.stac.set_link(Link::self_(path.to_string_lossy()));
        self.stac.assets_mut().clear();
        while let Some(result) = join_set.join_next().await {
            let (key, asset) = result?.await?;
            let _ = self.stac.assets_mut().insert(key, asset);
        }
        if self.write_stac {
            crate::write_json_to_path(path, serde_json::to_value(self.stac.clone())?).await?;
        }
        Ok(self.stac)
    }

    fn asset_downloaders(&self) -> impl Iterator<Item = AssetDownloader> {
        let client = self.client.clone();
        self.stac
            .assets()
            .clone()
            .into_iter()
            .map(move |(key, asset)| AssetDownloader {
                key,
                asset,
                client: client.clone(),
            })
    }
}

impl AssetDownloader {
    async fn download_to(mut self, directory: impl AsRef<Path>) -> Result<(String, Asset)> {
        let url = Url::parse(&self.asset.href)?;
        let file_name = url
            .path_segments()
            .and_then(|s| s.last().map(|s| s.to_string()))
            .unwrap_or_else(|| self.key.clone());
        let mut response = self
            .client
            .get(self.asset.href)
            .send()
            .await
            .and_then(|response| response.error_for_status())?;
        let path = directory.as_ref().join(file_name.clone());
        let mut file = File::create(path).await?;
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
        }
        self.asset.href = format!("./{}", file_name);
        Ok((self.key, self.asset))
    }
}

impl Download for Item {}
impl Download for Collection {}

#[cfg(test)]
mod tests {
    use super::Download;
    use mockito::Server;
    use stac::{Asset, Href, Item, Link, Links};
    use tempdir::TempDir;

    #[tokio::test]
    async fn download() {
        let mut server = Server::new_async().await;
        let download = server
            .mock("GET", "/asset.tif")
            .with_body("fake geotiff, sorry!")
            .create_async()
            .await;
        let mut item = Item::new("an-id");
        item.set_link(Link::collection("./collection.json"));
        let _ = item.assets.insert(
            "data".to_string(),
            Asset::new(format!("{}/asset.tif", server.url())),
        );
        item.set_href("http://stac-async-rs.test/item.json");
        let temp_dir = TempDir::new("download").unwrap();
        let item = item.download(temp_dir.path()).await.unwrap();
        download.assert_async().await;
        assert_eq!(
            item.link("canonical").unwrap().href,
            "http://stac-async-rs.test/item.json"
        );
        assert_eq!(
            item.collection_link().unwrap().href,
            "http://stac-async-rs.test/collection.json"
        );
        let path = temp_dir.path().join("item.json");
        assert_eq!(item.self_link().unwrap().href, path.to_string_lossy());
        for asset in item.assets.values() {
            assert!(temp_dir.path().join(&asset.href).exists());
        }
        let _: Item = crate::read(path.to_string_lossy()).await.unwrap();
    }
}
