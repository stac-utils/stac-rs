use super::{Input, Run};
use crate::{Result, Value};
use stac::{item::Builder, Asset};
use std::path::Path;
use tokio::sync::mpsc::Sender;
use url::Url;

/// Arguments for the `item` subcommand.
#[derive(clap::Args, Debug)]
pub(crate) struct Args {
    /// The item id or asset href
    pub(crate) id_or_href: String,

    /// The output file, if not provided the item will be printed to standard output
    pub(crate) outfile: Option<String>,

    /// The asset's key
    #[arg(short, long, default_value = "data")]
    pub(crate) key: String,

    /// Roles to use for the created asset
    #[arg(short, long = "role", default_values_t = ["data".to_string()])]
    pub(crate) roles: Vec<String>,

    /// Don't use GDAL to add geospatial metadata to the item
    #[cfg(feature = "gdal")]
    #[arg(long)]
    pub(crate) disable_gdal: bool,

    /// Allow assets to have relative hrefs
    #[arg(long)]
    pub(crate) allow_relative_hrefs: bool,
}

impl Run for Args {
    async fn run(self, _: Input, _: Option<Sender<Value>>) -> Result<Option<Value>> {
        let (id, href): (Option<String>, Option<String>) =
            if Url::parse(&self.id_or_href).is_err() && !Path::new(&self.id_or_href).exists() {
                (Some(self.id_or_href), None)
            } else {
                (None, Some(self.id_or_href))
            };
        let id = id
            .or_else(|| {
                Path::new(href.as_ref().expect("if id is none, href should exist"))
                    .file_stem()
                    .map(|s| s.to_string_lossy().to_string())
            })
            .unwrap_or_else(|| "default".to_string());
        let mut builder = Builder::new(id).canonicalize_paths(!self.allow_relative_hrefs);
        #[cfg(feature = "gdal")]
        {
            builder = builder.enable_gdal(!self.disable_gdal);
        }
        if let Some(href) = href {
            let mut asset = Asset::new(href);
            asset.roles = self.roles;
            builder = builder.asset(self.key, asset);
        }
        let item = builder.build()?;
        Ok(Some(stac::Value::from(item).into()))
    }

    fn take_outfile(&mut self) -> Option<String> {
        self.outfile.take()
    }
}
