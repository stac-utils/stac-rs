//! Command line parsing and execution.

// The verbosity stuff is cribbed from https://github.com/clap-rs/clap-verbosity-flag/blob/c621a6a8a7c0b6df8f1464a985a5d076b4915693/src/lib.rs and updated for tracing

use crate::{
    subcommand::{search, serve, translate, validate},
    Error, Result, Value,
};
use clap::{Parser, ValueEnum};
use stac::Format;
use std::{convert::Infallible, io::Write, str::FromStr};
use tokio::io::AsyncReadExt;
use tracing::metadata::Level;

/// Arguments, as parsed from the command line (usually).
#[derive(Parser, Debug, Clone)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The input format, if not provided will be inferred from the input file's extension, falling back to json
    #[arg(short, long, global = true)]
    pub input_format: Option<FormatWrapper>,

    /// key=value pairs to use for the input object store
    #[arg(long = "input-option")]
    pub input_options: Vec<KeyValue>,

    /// The output format, if not provided will be inferred from the output file's extension, falling back to json
    #[arg(short, long, global = true)]
    pub output_format: Option<FormatWrapper>,

    /// key=value pairs to use for the output object store
    #[arg(long = "output-option")]
    pub output_options: Vec<KeyValue>,

    /// key=value pairs to use for both the input and the output object store
    #[arg(short = 'c', long = "option")]
    pub options: Vec<KeyValue>,

    /// Stream the items to output as ndjson, default behavior is to return them all at the end of the operation
    #[arg(short, long)]
    pub stream: bool,

    #[arg(
        long,
        short = 'v',
        action = clap::ArgAction::Count,
        global = true,
        help = ErrorLevel::verbose_help(),
        long_help = ErrorLevel::verbose_long_help(),
    )]
    pub verbose: u8,

    #[arg(
        long,
        short = 'q',
        action = clap::ArgAction::Count,
        global = true,
        help = ErrorLevel::quiet_help(),
        long_help = ErrorLevel::quiet_long_help(),
        conflicts_with = "verbose",
    )]
    pub quiet: u8,

    /// The subcommand to run.
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

/// A subcommand.
#[derive(Debug, clap::Subcommand, Clone)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    /// Interact with a pgstac database
    #[cfg(feature = "pgstac")]
    Pgstac(crate::subcommand::pgstac::Args),

    /// Search for STAC items
    Search(search::Args),

    /// Serve a STAC API
    Serve(serve::Args),

    /// Translate STAC values between formats
    Translate(translate::Args),

    /// Validate a STAC object using json-schema
    Validate(validate::Args),
}

#[derive(Copy, Clone, Debug, Default)]
struct ErrorLevel;

impl Args {
    /// Returns the tracing log level for these args.
    pub fn log_level(&self) -> Option<Level> {
        level_enum(self.verbosity())
    }

    fn verbosity(&self) -> i8 {
        level_value(ErrorLevel::default()) - (self.quiet as i8) + (self.verbose as i8)
    }

    /// Runs whatever these arguments say that we should run.
    pub async fn run(self) -> Result<()> {
        match &self.subcommand {
            #[cfg(feature = "pgstac")]
            Subcommand::Pgstac(args) => self.pgstac(args).await,
            Subcommand::Search(args) => self.search(args).await,
            Subcommand::Serve(args) => self.serve(args).await,
            Subcommand::Translate(args) => self.translate(args).await,
            Subcommand::Validate(args) => self.validate(args).await,
        }
    }

    pub async fn get(&self, href: impl Into<Option<String>>) -> Result<stac::Value> {
        let mut href = href.into();
        if href.as_deref() == Some("-") {
            href = None;
        }
        let format = self
            .input_format
            .map(Into::into)
            .or(href.as_deref().and_then(Format::infer_from_href))
            .unwrap_or_default();
        if let Some(href) = href {
            format
                .get_opts(
                    href,
                    option_iter(&self.input_options).chain(option_iter(&self.options)),
                )
                .await
                .map_err(Error::from)
        } else {
            let mut buf = Vec::new();
            let _ = tokio::io::stdin().read_to_end(&mut buf).await?;
            format.from_bytes(buf).map_err(Error::from)
        }
    }

    pub async fn put(&self, value: impl Into<Value>, href: impl Into<Option<&str>>) -> Result<()> {
        let mut href = href.into();
        if href == Some("-") {
            href = None;
        }
        let format = self
            .output_format
            .map(Into::into)
            .or(href.and_then(Format::infer_from_href))
            .unwrap_or_default();
        let value = value.into();
        if let Some(href) = href {
            let put_result = format
                .put_opts(
                    href,
                    value,
                    option_iter(&self.output_options).chain(option_iter(&self.options)),
                )
                .await?;
            if let Some(put_result) = put_result {
                if put_result.e_tag.is_some() || put_result.version.is_some() {
                    tracing::info!(
                        "put result: e_tag={}, version={}",
                        put_result.e_tag.as_deref().unwrap_or("None"),
                        put_result.version.as_deref().unwrap_or("None")
                    );
                } else {
                    tracing::info!("put ok");
                }
            }
        } else {
            let mut bytes = format.into_vec(value)?;
            bytes.push(b'\n');
            std::io::stdout().write_all(&bytes)?;
        }
        Ok(())
    }

    pub async fn load(
        &self,
        backend: &mut impl stac_server::Backend,
        hrefs: impl Iterator<Item = &str>,
        load_collection_items: bool,
        create_collections: bool,
    ) -> Result<Load> {
        use stac::{Collection, Links, Value};
        use std::collections::{HashMap, HashSet};
        use tokio::task::JoinSet;

        let mut join_set = JoinSet::new();
        let mut read_from_stdin = false;
        for href in hrefs {
            if href == "-" {
                read_from_stdin = true;
            } else {
                let args = self.clone();
                let href = href.to_string();
                let _ = join_set.spawn(async move { args.get(Some(href)).await });
            }
        }
        if read_from_stdin {
            let args = self.clone();
            let _ = join_set.spawn(async move { args.get(None).await });
        }
        let mut item_join_set = JoinSet::new();
        let mut items: HashMap<Option<String>, Vec<_>> = HashMap::new();
        let mut collections = HashSet::new();
        let mut load = Load::default();
        while let Some(result) = join_set.join_next().await {
            let value = result??;
            match value {
                Value::Collection(mut collection) => {
                    if load_collection_items {
                        collection.make_links_absolute()?;
                        for link in collection.iter_item_links() {
                            let args = self.clone();
                            let href = link.href.to_string().clone();
                            let _ = item_join_set.spawn(async move { args.get(href).await });
                        }
                    }
                    let _ = collections.insert(collection.id.clone());
                    collection.remove_structural_links();
                    load.collections += 1;
                    backend.add_collection(collection).await?;
                }
                Value::Item(item) => {
                    items.entry(item.collection.clone()).or_default().push(item);
                }
                Value::ItemCollection(item_collection) => {
                    for item in item_collection.items {
                        items.entry(item.collection.clone()).or_default().push(item);
                    }
                }
                Value::Catalog(catalog) => {
                    tracing::warn!("skipping catalog: {}", catalog.id);
                }
            }
        }
        while let Some(result) = item_join_set.join_next().await {
            let value = result??;
            if let Value::Item(item) = value {
                items.entry(item.collection.clone()).or_default().push(item);
            } else {
                tracing::warn!(
                    "skipping {} that was behind an item link",
                    value.type_name()
                );
            }
        }
        for (collection, items) in items {
            if let Some(collection) = collection {
                if collections.contains(&collection) {
                    load.items += items.len();
                    backend.add_items(items).await?;
                } else if create_collections {
                    tracing::info!(
                        "creating collection={} for {} item(s)",
                        collection,
                        items.len()
                    );
                    let collection = Collection::from_id_and_items(collection, &items);
                    load.collections += 1;
                    backend.add_collection(collection).await?;
                    load.items += items.len();
                    backend.add_items(items).await?;
                } else {
                    tracing::warn!(
                        "skipping {} item(s) with collection {}",
                        items.len(),
                        collection
                    );
                }
            } else if create_collections {
                tracing::info!(
                    "creating auto-generated collection for {} item(s)",
                    items.len()
                );
                let collection = Collection::from_id_and_items("auto-generated", &items);
                load.collections += 1;
                backend.add_collection(collection).await?;
                load.items += items.len();
                backend.add_items(items).await?;
            } else {
                tracing::warn!("skipping {} item(s) without a collection", items.len());
            }
        }
        Ok(load)
    }
}

impl ErrorLevel {
    fn default() -> Option<Level> {
        Some(Level::ERROR)
    }

    fn verbose_help() -> Option<&'static str> {
        Some("Increase verbosity")
    }

    fn verbose_long_help() -> Option<&'static str> {
        None
    }

    fn quiet_help() -> Option<&'static str> {
        Some("Decrease verbosity")
    }

    fn quiet_long_help() -> Option<&'static str> {
        None
    }
}

fn level_enum(verbosity: i8) -> Option<Level> {
    match verbosity {
        i8::MIN..=-1 => None,
        0 => Some(Level::ERROR),
        1 => Some(Level::WARN),
        2 => Some(Level::INFO),
        3 => Some(Level::DEBUG),
        4..=i8::MAX => Some(Level::TRACE),
    }
}

fn level_value(level: Option<Level>) -> i8 {
    match level {
        None => -1,
        Some(Level::ERROR) => 0,
        Some(Level::WARN) => 1,
        Some(Level::INFO) => 2,
        Some(Level::DEBUG) => 3,
        Some(Level::TRACE) => 4,
    }
}

/// A collection of configuration entries.
/// `key=value`` pairs for object store configuration
#[derive(Clone, Debug)]
pub struct KeyValue {
    key: String,
    value: String,
}

impl FromStr for KeyValue {
    type Err = Infallible;
    fn from_str(s: &str) -> std::result::Result<Self, Infallible> {
        if let Some((key, value)) = s.split_once('=') {
            Ok(KeyValue {
                key: key.to_string(),
                value: value.to_string(),
            })
        } else {
            Ok(KeyValue {
                key: s.to_string(),
                value: String::new(),
            })
        }
    }
}

fn option_iter(options: &[KeyValue]) -> impl Iterator<Item = (&str, &str)> {
    options
        .iter()
        .map(|kv| (kv.key.as_str(), kv.value.as_str()))
}

/// The counts from a load
#[derive(Debug, Default)]
pub struct Load {
    pub collections: usize,
    pub items: usize,
}

#[derive(Debug, Copy, Clone, ValueEnum)]
pub enum FormatWrapper {
    Json,
    Ndjson,
    Geoparquet,
}

impl From<Format> for FormatWrapper {
    fn from(format: Format) -> Self {
        match format {
            Format::Json(_) => Self::Json,
            Format::NdJson => Self::Ndjson,
            Format::Geoparquet(_) => Self::Geoparquet,
        }
    }
}

impl From<FormatWrapper> for Format {
    fn from(wrapper: FormatWrapper) -> Self {
        match wrapper {
            FormatWrapper::Json => Format::json(),
            FormatWrapper::Ndjson => Format::ndjson(),
            FormatWrapper::Geoparquet => Format::geoparquet(),
        }
    }
}
