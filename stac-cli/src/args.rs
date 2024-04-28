use crate::{Error, Result, Subcommand};
use clap::Parser;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::json;
use stac::{item::Builder, Asset, Value};
use stac_api::{GetItems, GetSearch, Item, ItemCollection};
use stac_async::ApiClient;
use stac_validate::Validate;
use std::path::Path;
use tokio_stream::StreamExt;
use url::Url;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Use a compact representation of the output, if possible.
    #[arg(short, long)]
    compact: bool,

    /// The subcommand to run.
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

impl Args {
    pub async fn execute(self) -> i32 {
        use Subcommand::*;
        let result = match &self.subcommand {
            Item {
                id_or_href,
                id,
                key,
                role,
                allow_relative_paths,
                disable_gdal,
                collect,
            } => self.item(
                id_or_href,
                id.as_deref(),
                key,
                role,
                *allow_relative_paths,
                *disable_gdal,
                *collect,
            ),
            Search {
                href,
                max_items,
                limit,
                bbox,
                datetime,
                intersects,
                ids,
                collections,
                fields,
                sortby,
                filter_crs,
                filter_lang,
                filter,
                stream,
            } => {
                self.api_search(
                    href,
                    *max_items,
                    limit,
                    bbox,
                    datetime,
                    intersects,
                    ids,
                    collections,
                    fields,
                    sortby,
                    filter_crs,
                    filter_lang,
                    filter,
                    *stream,
                )
                .await
            }
            Sort { href } => self.sort(href.as_deref()).await,
            Validate { href } => self.validate(href.as_deref()).await,
        };
        match result {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("ERROR: {}", err);
                err.code()
            }
        }
    }

    fn item(
        &self,
        href_or_id: &str,
        id: Option<&str>,
        key: &str,
        roles: &[String],
        allow_relative_paths: bool,
        mut disable_gdal: bool,
        collect: bool,
    ) -> Result<()> {
        if !cfg!(feature = "gdal") {
            tracing::info!(disable_gdal = true, "gdal feature not enabled");
            disable_gdal = true;
        }
        let mut href = None;
        let id = if let Ok(url) = Url::parse(href_or_id) {
            href = Some(href_or_id);
            id.map(|id| id.to_string()).unwrap_or_else(|| {
                url.path_segments()
                    .and_then(|path_segments| path_segments.last())
                    .and_then(|path_segment| Path::new(path_segment).file_stem())
                    .map(|file_stem| file_stem.to_string_lossy().into_owned())
                    .unwrap_or_else(|| href_or_id.to_string())
            })
        } else {
            let path = Path::new(href_or_id);
            if path.exists() {
                href = Some(href_or_id);
                id.map(|id| id.to_string()).unwrap_or_else(|| {
                    path.file_stem()
                        .map(|file_stem| file_stem.to_string_lossy().into_owned())
                        .unwrap_or_else(|| href_or_id.to_string())
                })
            } else {
                href_or_id.to_string()
            }
        };
        let mut builder = Builder::new(id)
            .enable_gdal(!disable_gdal)
            .canonicalize_paths(!allow_relative_paths);
        if let Some(href) = href {
            let mut asset = Asset::new(href);
            asset.roles = roles.to_vec();
            builder = builder.asset(key, asset);
        }
        let item = builder.into_item()?;
        if collect {
            let value = serde_json::from_reader(std::io::stdin())?;
            match value {
                Value::Item(stdin_item) => {
                    self.println(stac::ItemCollection::from(vec![stdin_item, item]))
                }
                Value::ItemCollection(mut item_collection) => {
                    item_collection.items.push(item);
                    self.println(item_collection)
                }
                Value::Catalog(_) | Value::Collection(_) => Err(Error::Custom(format!(
                    "unexpected STAC object type on standard input: {}",
                    value.type_name()
                ))),
            }
        } else {
            self.println(item)
        }
    }

    async fn api_search(
        &self,
        href: &str,
        max_items: Option<usize>,
        limit: &Option<String>,
        bbox: &Option<String>,
        datetime: &Option<String>,
        intersects: &Option<String>,
        ids: &Option<String>,
        collections: &Option<String>,
        fields: &Option<String>,
        sortby: &Option<String>,
        filter_crs: &Option<String>,
        filter_lang: &Option<String>,
        filter: &Option<String>,
        stream: bool,
    ) -> Result<()> {
        let get_items = GetItems {
            limit: limit.clone(),
            bbox: bbox.clone(),
            datetime: datetime.clone(),
            fields: fields.clone(),
            sortby: sortby.clone(),
            filter_crs: filter_crs.clone(),
            filter_lang: filter_lang.clone(),
            filter: filter.clone(),
            additional_fields: Default::default(),
        };
        let get_search = GetSearch {
            intersects: intersects.clone(),
            ids: ids.clone(),
            collections: collections.clone(),
            items: get_items,
        };
        let search = get_search.try_into()?;
        let client = ApiClient::new(href)?;
        let item_stream = client.search(search).await?;
        tokio::pin!(item_stream);
        let mut num_items = 0;
        if stream {
            while let Some(result) = item_stream.next().await {
                let item: Item = result?;
                num_items += 1;
                self.println_compact(item)?;
                if max_items
                    .map(|max_items| num_items >= max_items)
                    .unwrap_or(false)
                {
                    break;
                }
            }
        } else {
            let mut items = Vec::new();
            while let Some(result) = item_stream.next().await {
                num_items += 1;
                items.push(result?);
                if max_items
                    .map(|max_items| num_items >= max_items)
                    .unwrap_or(false)
                {
                    break;
                }
            }
            let item_collection = ItemCollection::new(items)?;
            self.println(item_collection)?;
        }
        Ok(())
    }

    async fn sort(&self, href: Option<&str>) -> Result<()> {
        let value: Value = self.read_href(href).await?;
        self.println(value)
    }

    async fn validate(&self, href: Option<&str>) -> Result<()> {
        let value: serde_json::Value = self.read_href(href).await?;
        let mut errors: Vec<serde_json::Value> = Vec::new();
        let mut update_errors = |result: std::result::Result<(), stac_validate::Error>| match result
        {
            Ok(()) => {}
            Err(err) => match err {
                stac_validate::Error::Validation(ref e) => {
                    errors.extend(e.iter().map(|error| {
                        json!({
                                "type": "validation",
                                "instance_path": error.instance_path,
                                "schema_path": error.schema_path,
                                "description": error.to_string(),
                        })
                    }));
                }
                _ => errors.push(json!({
                    "type": "other",
                    "message": err.to_string(),
                })),
            },
        };
        if let Some(collections) = value
            .as_object()
            .and_then(|object| object.get("collections"))
        {
            if let Some(collections) = collections.as_array() {
                for collection in collections.iter() {
                    let collection = collection.clone();
                    let result = tokio::task::spawn_blocking(move || collection.validate()).await?;
                    update_errors(result);
                }
            } else {
                return Err(Error::Custom(
                    "expected the 'collections' key to be an array".to_string(),
                ));
            }
        } else {
            let result = tokio::task::spawn_blocking(move || value.validate()).await?;
            update_errors(result);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            self.println(errors)?;
            Err(Error::Custom(format!(
                "one or more errors during validation"
            )))
        }
    }

    async fn read_href<D: DeserializeOwned>(&self, href: Option<&str>) -> Result<D> {
        if let Some(href) = href {
            stac_async::read_json(href).await.map_err(Error::from)
        } else {
            serde_json::from_reader(std::io::stdin()).map_err(Error::from)
        }
    }

    fn println_compact<S: Serialize>(&self, s: S) -> Result<()> {
        Ok(println!("{}", serde_json::to_string(&s)?))
    }

    fn println<S: Serialize>(&self, s: S) -> Result<()> {
        let output = if self.compact {
            serde_json::to_string(&s)?
        } else {
            serde_json::to_string_pretty(&s)?
        };
        Ok(println!("{}", output))
    }
}
