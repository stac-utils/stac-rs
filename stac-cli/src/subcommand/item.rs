use crate::{Error, ItemArgs, Result, Subcommand};
use stac::{item::Builder, Asset, Value};
use std::path::Path;
use url::Url;

impl Subcommand {
    pub(crate) fn item(args: ItemArgs) -> Result<Value> {
        let mut disable_gdal = args.disable_gdal;
        if !(disable_gdal || cfg!(feature = "gdal")) {
            tracing::info!(disable_gdal = true, "gdal feature not enabled");
            disable_gdal = true;
        }
        let mut href = None;
        let id = if let Ok(url) = Url::parse(&args.id_or_href) {
            href = Some(args.id_or_href.clone());
            args.id
                .as_ref()
                .map(|id| id.to_string())
                .unwrap_or_else(|| {
                    url.path_segments()
                        .and_then(|path_segments| path_segments.last())
                        .and_then(|path_segment| Path::new(path_segment).file_stem())
                        .map(|file_stem| file_stem.to_string_lossy().into_owned())
                        .unwrap_or_else(|| args.id_or_href.to_string())
                })
        } else {
            let path = Path::new(&args.id_or_href);
            if path.exists() {
                href = Some(args.id_or_href.clone());
                args.id
                    .as_ref()
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| {
                        path.file_stem()
                            .map(|file_stem| file_stem.to_string_lossy().into_owned())
                            .unwrap_or_else(|| args.id_or_href.to_string())
                    })
            } else {
                args.id_or_href.to_string()
            }
        };
        let mut builder = Builder::new(id)
            .enable_gdal(!disable_gdal)
            .canonicalize_paths(!args.allow_relative_paths);
        if let Some(href) = href {
            let mut asset = Asset::new(href);
            asset.roles = args.role.to_vec();
            builder = builder.asset(&args.key, asset);
        }
        let item = builder.into_item()?;
        if args.collect {
            let value = serde_json::from_reader(std::io::stdin())?;
            match value {
                Value::Item(stdin_item) => {
                    Ok(stac::ItemCollection::from(vec![stdin_item, item]).into())
                }
                Value::ItemCollection(mut item_collection) => {
                    item_collection.items.push(item);
                    Ok(item_collection.into())
                }
                Value::Catalog(_) | Value::Collection(_) => Err(Error::Custom(format!(
                    "unexpected STAC object type on standard input: {}",
                    value.type_name()
                ))),
            }
        } else {
            Ok(item.into())
        }
    }
}
