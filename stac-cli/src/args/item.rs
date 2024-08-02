use crate::{Error, Printer, Result};
use clap::Args;
use stac::{item::Builder, Asset, Value};
use std::path::Path;
use url::Url;

/// Arguments for creating an item.
#[derive(Args, Debug)]
pub struct ItemArgs {
    /// The item id or asset href.
    id_or_href: String,

    /// The item id, if the positional argument is an href.
    ///
    /// If not provided, will be inferred from the filename in the href.
    #[arg(short, long)]
    id: Option<String>,

    /// The asset key, if the positional argument is an href.
    #[arg(short, long, default_value = "data")]
    key: String,

    /// The asset roles, if the positional argument is an href.
    ///
    /// Can be provided multiple times.
    #[arg(short, long)]
    role: Vec<String>,

    /// Allow relative paths.
    ///
    /// If false, all path will be canonicalized, which requires that the
    /// files actually exist on the filesystem.
    #[arg(long)]
    allow_relative_paths: bool,

    /// Don't use GDAL for item creation, if the positional argument is an href.
    ///
    /// Automatically set to true if this crate is compiled without GDAL.
    #[arg(long)]
    disable_gdal: bool,

    /// Collect an item or item collection from standard input, and add the
    /// newly created to it into a new item collection.
    #[arg(short, long)]
    collect: bool,
}

impl ItemArgs {
    /// Creates a [stac::Item].
    pub fn execute(&self, printer: Printer) -> Result<()> {
        let mut disable_gdal = self.disable_gdal;
        if !(disable_gdal || cfg!(feature = "gdal")) {
            tracing::info!(disable_gdal = true, "gdal feature not enabled");
            disable_gdal = true;
        }
        let mut href = None;
        let id = if let Ok(url) = Url::parse(&self.id_or_href) {
            href = Some(self.id_or_href.clone());
            self.id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| {
                    url.path_segments()
                        .and_then(|path_segments| path_segments.last())
                        .and_then(|path_segment| Path::new(path_segment).file_stem())
                        .map(|file_stem| file_stem.to_string_lossy().into_owned())
                        .unwrap_or_else(|| self.id_or_href.to_string())
                })
        } else {
            let path = Path::new(&self.id_or_href);
            if path.exists() {
                href = Some(self.id_or_href.clone());
                self.id
                    .as_ref()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| {
                        path.file_stem()
                            .map(|file_stem| file_stem.to_string_lossy().into_owned())
                            .unwrap_or_else(|| self.id_or_href.to_string())
                    })
            } else {
                self.id_or_href.to_string()
            }
        };
        let mut builder = Builder::new(id)
            .enable_gdal(!disable_gdal)
            .canonicalize_paths(!self.allow_relative_paths);
        if let Some(href) = href {
            let mut asset = Asset::new(href);
            asset.roles = self.role.to_vec();
            builder = builder.asset(&self.key, asset);
        }
        let item = builder.into_item()?;
        if self.collect {
            let value = serde_json::from_reader(std::io::stdin())?;
            match value {
                Value::Item(stdin_item) => {
                    printer.println(stac::ItemCollection::from(vec![stdin_item, item]))
                }
                Value::ItemCollection(mut item_collection) => {
                    item_collection.items.push(item);
                    printer.println(item_collection)
                }
                Value::Catalog(_) | Value::Collection(_) => Err(Error::Custom(format!(
                    "unexpected STAC object type on standard input: {}",
                    value.type_name()
                ))),
            }
        } else {
            printer.println(item)
        }
    }
}
