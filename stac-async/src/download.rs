//! Download assets.

use crate::{Error, Result};
use async_trait::async_trait;
use reqwest::Client;
use serde::Serialize;
use stac::{Asset, Assets, Collection, Href, Item, Link, Links, Value};
use std::path::{Path, PathBuf};
use tokio::{fs::File, io::AsyncWriteExt, sync::mpsc::Sender, task::JoinSet};
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
    let value = crate::read(href).await?;
    match value {
        Value::Item(item) => item.download(directory).await.map(|item| Value::Item(item)),
        Value::Collection(collection) => collection
            .download(directory)
            .await
            .map(|collection| Value::Collection(collection)),
        _ => Err(Error::CannotDownload(value)),
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
    sender: Option<Sender<Message>>,
    write_stac: bool,
}

/// A message from a downloader.
#[derive(Debug)]
pub enum Message {
    /// Create a download directory.
    CreateDirectory(PathBuf),

    /// Send a GET request for an asset.
    GetAsset {
        /// The asset downloader id.
        id: usize,

        /// The url of the asset.
        url: Url,
    },

    /// Got a successful asset response.
    GotAsset {
        /// The asset downloader id.
        id: usize,

        /// The length of the asset response.
        content_length: Option<u64>,
    },

    /// Update the number of bytes written.
    Update {
        /// The asset downloader id.
        id: usize,

        /// Then umber of bytes written.
        bytes_written: usize,
    },

    /// The download is finished.
    FinishedDownload(usize),

    /// All downloads are finished.
    FinishedAllDownloads,

    /// Write the stac file to the local filesystem.
    WriteStac(PathBuf),
}

#[derive(Debug)]
struct AssetDownloader {
    id: usize,
    key: String,
    asset: Asset,
    client: Client,
    sender: Option<Sender<Message>>,
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
            sender: None,
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

    /// Set the sender for messages from this downloader.
    ///
    /// # Examples
    ///
    /// TODO
    pub fn with_sender(mut self, sender: Sender<Message>) -> Downloader<T> {
        self.sender = Some(sender);
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
            self.send(|| Message::CreateDirectory(directory.to_path_buf()))
                .await?;
            tokio::fs::create_dir_all(directory).await?;
        }
        for asset_downloader in self.asset_downloaders() {
            let directory = directory.to_path_buf();
            let _ = join_set.spawn(async move { asset_downloader.download(directory) });
        }
        let path = directory.join(&self.file_name);
        self.stac.set_link(Link::self_(path.to_string_lossy()));
        while let Some(result) = join_set.join_next().await {
            // TODO we should allow some assets to gracefully fail, maybe?
            let (key, asset) = result?.await?;
            let _ = self.stac.assets_mut().insert(key, asset);
        }
        self.send(|| Message::FinishedAllDownloads).await?;
        if self.write_stac {
            self.send(|| Message::WriteStac(path.clone())).await?;
            crate::write_json_to_path(path, serde_json::to_value(self.stac.clone())?).await?;
        }
        Ok(self.stac)
    }

    fn asset_downloaders(&mut self) -> Vec<AssetDownloader> {
        self.stac
            .assets_mut()
            .drain()
            .enumerate()
            .map(|(id, (key, asset))| AssetDownloader {
                id,
                key,
                asset,
                client: self.client.clone(),
                sender: self.sender.clone(),
            })
            .collect()
    }

    async fn send(&mut self, f: impl Fn() -> Message) -> Result<()> {
        if let Some(sender) = &mut self.sender {
            sender.send(f()).await.map_err(Error::from)
        } else {
            Ok(())
        }
    }
}

impl AssetDownloader {
    async fn download(mut self, directory: impl AsRef<Path>) -> Result<(String, Asset)> {
        let id = self.id;
        let url = Url::parse(&self.asset.href)?;
        let file_name = url
            .path_segments()
            .and_then(|s| s.last().map(|s| s.to_string()))
            .unwrap_or_else(|| self.key.clone());
        self.send(|| Message::GetAsset {
            id,
            url: url.clone(),
        })
        .await?;
        let mut response = self
            .client
            .get(&self.asset.href)
            .send()
            .await
            .and_then(|response| response.error_for_status())?;
        let content_length = response.content_length();
        self.send(|| Message::GotAsset { id, content_length })
            .await?;
        let path = directory.as_ref().join(file_name.clone());
        let mut file = File::create(path).await?;
        let mut bytes_written = 0;
        while let Some(chunk) = response.chunk().await? {
            file.write_all(&chunk).await?;
            bytes_written += chunk.len();
            self.try_send(|| Message::Update { id, bytes_written });
        }
        self.try_send(|| Message::FinishedDownload(id));
        self.asset.href = format!("./{}", file_name);
        Ok((self.key, self.asset))
    }

    async fn send(&mut self, f: impl Fn() -> Message) -> Result<()> {
        if let Some(sender) = &mut self.sender {
            sender.send(f()).await.map_err(Error::from)
        } else {
            Ok(())
        }
    }

    /// Sometimes we want to send without erroring, e.g. during a download when
    /// the buffer might fill up.
    fn try_send(&mut self, f: impl Fn() -> Message) {
        if let Some(sender) = &mut self.sender {
            let _ = sender.try_send(f());
        }
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
