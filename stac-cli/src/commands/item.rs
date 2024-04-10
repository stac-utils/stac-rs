use crate::Result;
use stac::{item::Builder, Asset};

pub fn item(
    id: String,
    href: String,
    key: String,
    roles: Vec<String>,
    allow_relative_paths: bool,
    compact: bool,
    disable_gdal: bool,
) -> Result<()> {
    let mut asset = Asset::new(href);
    asset.roles = roles;
    let item = Builder::new(id)
        .asset(key, asset)
        .canonicalize_paths(!allow_relative_paths)
        .enable_gdal(!disable_gdal)
        .into_item()?;
    if compact {
        println!("{}", serde_json::to_string(&item)?);
    } else {
        println!("{}", serde_json::to_string_pretty(&item)?);
    }
    Ok(())
}
