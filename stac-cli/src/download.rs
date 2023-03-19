use crate::Result;
use console::Term;
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use serde::Serialize;
use stac::{Assets, Href, Links};
use stac_async::download::{Downloader, Message};
use std::{collections::HashMap, path::PathBuf};
use tokio::sync::mpsc::{self, Receiver};
use url::Url;

const BUFFER: usize = 50;

pub async fn download<A>(stac: A, directory: PathBuf, create_directory: bool) -> Result<()>
where
    A: Assets + Href + Links + Serialize + Clone + Send + Sync + 'static,
{
    use Message::*;
    use Progress::*;

    let (tx, mut rx) = mpsc::channel(BUFFER);
    let result = {
        let directory = directory.clone();
        tokio::spawn(async move {
            Downloader::new(stac)?
                .create_directory(create_directory)
                .with_sender(tx)
                .download(directory)
                .await
        })
    };

    let term = Term::stderr();
    let multi_progress = MultiProgress::new();
    let progress_style = ProgressStyle::with_template("{spinner:.green} {msg} [{elapsed_precise}] [{wide_bar}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})").unwrap().progress_chars("#>-");
    let mut senders = HashMap::new();
    while let Some(message) = rx.recv().await {
        match message {
            CreateDirectory(path) => {
                term.write_line(&format!("Creating output directory {}", path.display()))?;
            }
            GetAsset { id, url } => {
                let progress_bar = multi_progress.add(ProgressBar::new(0));
                progress_bar.set_style(progress_style.clone());
                let (tx, rx) = mpsc::channel(BUFFER);
                tokio::spawn(async move { progress(progress_bar, rx, url).await });
                senders.insert(id, tx);
            }
            GotAsset { id, content_length } => {
                if let Some((tx, content_length)) = senders
                    .get(&id)
                    .and_then(|tx| content_length.map(|c| (tx, c)))
                {
                    tx.send(ContentLength(content_length)).await?;
                }
            }
            Update { id, bytes_written } => {
                if let Some((tx, position)) = senders
                    .get(&id)
                    .and_then(|tx| u64::try_from(bytes_written).ok().map(|p| (tx, p)))
                {
                    tx.send(Position(position)).await?;
                }
            }
            FinishedDownload(id) => {
                if let Some(tx) = senders.remove(&id) {
                    tx.send(Done).await?;
                }
            }
            FinishedAllDownloads => {
                multi_progress.clear()?;
                term.write_line(&format!("Assets downloaded to {}", directory.display()))?;
            }
            WriteStac(path) => {
                term.write_line(&format!("Writing STAC to {}", path.display()))?;
            }
        }
    }
    let _ = result.await?;

    Ok(())
}

#[derive(Debug)]
pub enum Progress {
    ContentLength(u64),
    Position(u64),
    Done,
}

async fn progress(progress_bar: ProgressBar, mut rx: Receiver<Progress>, url: Url) {
    use Progress::*;
    progress_bar.set_message(url.to_string());
    while let Some(message) = rx.recv().await {
        match message {
            ContentLength(content_length) => {
                progress_bar.set_length(content_length);
            }
            Position(position) => {
                progress_bar.set_position(position);
            }
            Done => progress_bar.finish_and_clear(),
        }
    }
}
