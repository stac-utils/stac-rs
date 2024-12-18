use stac::{item::Builder as ItemBuilder, Asset, Value};
#[cfg(feature = "gdal")]
use stac_gdal::update_item;
use std::path::Path;

use crate::Result;

/// Arguments for the `create_item` subcommand.
#[derive(clap::Args, Debug, Clone)]
pub struct Args {
    /// The input file.
    // ///
    // /// If not provided or `-`, the input will be read from standard input.
    href: String,

    /// Asset key
    #[arg(default_value = "data")]
    asset_key: String,

    /// Semantic roles of the asset
    #[arg(short, long)]
    roles: Option<String>,
}

impl crate::Args {
    pub async fn create(&self, args: &Args) -> Result<()> {
        // TODO: Filename must be present or we need to react
        let filename = Path::new(&args.href)
            .file_name()
            .and_then(|s| s.to_str())
            .expect("Filename must be present");
        let mut asset = Asset::new(&args.href);
        if let Some(roles) = &args.roles {
            asset = asset.role(roles);
        }

        #[allow(unused_mut)] // we need this for gdal's `update_item` below
        let mut item = ItemBuilder::new(filename)
            .asset(&args.asset_key, asset)
            .build()?;

        #[cfg(feature = "gdal")]
        update_item(&mut item, false, true)?;

        self.put(Value::from(item), None).await?;
        Ok(())
    }
}
