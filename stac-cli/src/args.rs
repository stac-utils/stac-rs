use crate::{Error, Format, Result, Subcommand};
use clap::Parser;
use serde::Serialize;
use serde_json::json;
use stac::{item::Builder, Asset, Value};
use stac_api::{GetItems, GetSearch, Item, ItemCollection};
use stac_async::ApiClient;
use stac_server::{Api, Backend, MemoryBackend};
use stac_validate::Validate;
use std::{fs::File, io::Write, path::Path};
use tokio::net::TcpListener;
use tokio_stream::StreamExt;
use url::Url;

/// CLI arguments.
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
    /// Executes the subcommand.
    pub async fn execute(self) -> i32 {
        use Subcommand::*;
        let result = match &self.subcommand {
            Convert {
                from,
                to,
                in_format,
                out_format,
            } => {
                self.convert(from.as_deref(), to.as_deref(), *in_format, *out_format)
                    .await
            }
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
            Serve { href, pgstac } => self.serve(href, pgstac.as_deref()).await,
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

    async fn convert(
        &self,
        from: Option<&str>,
        to: Option<&str>,
        in_format: Option<Format>,
        out_format: Option<Format>,
    ) -> Result<()> {
        self.write_href(self.read_href(from, in_format).await?, to, out_format)
            .await
    }

    #[allow(clippy::too_many_arguments)]
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

    #[allow(clippy::too_many_arguments)]
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

    #[allow(unused_variables)] // for `pgstac` if we don't compile with it
    async fn serve(&self, hrefs: &[String], pgstac: Option<&str>) -> Result<()> {
        let root = "http://127.0.0.1:7822";
        let addr = "127.0.0.1:7822";
        if let Some(pgstac) = pgstac {
            #[cfg(feature = "pgstac")]
            {
                let mut backend = stac_server::PgstacBackend::new_from_stringlike(pgstac).await?;
                if !hrefs.is_empty() {
                    backend.add_from_hrefs(hrefs).await?;
                }
                let api = Api::new(backend, root)?;
                let router = stac_server::routes::from_api(api);
                let listener = TcpListener::bind(addr).await.unwrap();
                println!("Serving a STAC API at {} using a pgstac backend", root);
                axum::serve(listener, router).await.unwrap();
            }
            #[cfg(not(feature = "pgstac"))]
            return Err(Error::Custom(
                "stac-cli is not compiled with pgstac support".to_string(),
            ));
        } else {
            let mut backend = MemoryBackend::new();
            if !hrefs.is_empty() {
                backend.add_from_hrefs(hrefs).await?;
            }
            let api = Api::new(backend, root)?;
            let router = stac_server::routes::from_api(api);
            let listener = TcpListener::bind(addr).await.unwrap();
            println!("Serving a STAC API at {} using a memory backend", root);
            axum::serve(listener, router).await.unwrap();
        };
        Ok(())
    }

    async fn sort(&self, href: Option<&str>) -> Result<()> {
        // TODO allow specifying formats
        let value: Value = self.read_href(href, None).await?;
        self.println(value)
    }

    async fn validate(&self, href: Option<&str>) -> Result<()> {
        // TODO allow specifying formats
        let value: serde_json::Value = if let Some(href) = href {
            stac_async::read_json(href).await?
        } else {
            serde_json::from_reader(std::io::stdin())?
        };
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
            Err(Error::Custom(
                "one or more errors during validation".to_string(),
            ))
        }
    }

    async fn read_href(&self, href: Option<&str>, format: Option<Format>) -> Result<Value> {
        let format = format.unwrap_or_else(|| href.and_then(Format::from_href).unwrap_or_default());
        match format {
            Format::Json => {
                if let Some(href) = href {
                    stac_async::read_json(href).await.map_err(Error::from)
                } else {
                    serde_json::from_reader(std::io::stdin()).map_err(Error::from)
                }
            }
            #[cfg(feature = "parquet")]
            Format::GeoParquet => {
                let geo_table = if let Some(href) = href {
                    let file = File::open(href)?;
                    geoarrow::io::parquet::read_geoparquet(file, Default::default())?
                } else {
                    // FIXME
                    unimplemented!()
                };
                let items = stac_arrow::geo_table_to_items(geo_table)?;
                Ok(Value::ItemCollection(items.into()))
            }
        }
    }

    async fn write_href(
        &self,
        value: Value,
        href: Option<&str>,
        format: Option<Format>,
    ) -> Result<()> {
        let format = format.unwrap_or_else(|| href.and_then(Format::from_href).unwrap_or_default());
        match format {
            Format::Json => {
                if let Some(href) = href {
                    let output = if self.compact {
                        serde_json::to_string(&value)?
                    } else {
                        serde_json::to_string_pretty(&value)?
                    };
                    let mut file = File::create(href)?;
                    file.write_all(output.as_bytes())?;
                } else {
                    self.println(value)?;
                }
                Ok(())
            }
            #[cfg(feature = "parquet")]
            Format::GeoParquet => {
                let items = match value {
                    Value::ItemCollection(item_collection) => item_collection.items,
                    Value::Item(item) => vec![item],
                    _ => {
                        return Err(Error::Custom(format!(
                            "cannot write STAC GeoParquet of type: {}",
                            value.type_name()
                        )))
                    }
                };
                // TODO allow customizing batch size
                let mut geo_table = stac_arrow::items_to_geo_table(items)?;
                if let Some(href) = href {
                    let file = File::create(href)?;
                    geoarrow::io::parquet::write_geoparquet(&mut geo_table, file, None)?;
                } else {
                    geoarrow::io::parquet::write_geoparquet(
                        &mut geo_table,
                        std::io::stdout(),
                        None,
                    )?;
                }
                Ok(())
            }
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
