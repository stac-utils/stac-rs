use crate::{Output, Result, SearchArgs, Subcommand};
use stac_api::{GetItems, GetSearch, Item, ItemCollection};
use stac_async::ApiClient;
use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;

impl Subcommand {
    pub(crate) async fn search(args: SearchArgs, sender: Sender<Output>) -> Result<()> {
        let get_items = GetItems {
            limit: args.limit,
            bbox: args.bbox,
            datetime: args.datetime,
            fields: args.fields,
            sortby: args.sortby,
            filter_crs: args.filter_crs,
            filter_lang: args.filter_lang,
            filter: args.filter,
            additional_fields: Default::default(),
        };
        let get_search = GetSearch {
            intersects: args.intersects,
            ids: args.ids,
            collections: args.collections,
            items: get_items,
        };
        let search = get_search.try_into()?;
        if args
            .duckdb
            .unwrap_or_else(|| stac::geoparquet::has_extension(&args.href))
        {
            #[cfg(feature = "duckdb")]
            {
                let client = stac_duckdb::Client::from_href(args.href)?;
                let items = client.search_to_json(search)?;
                if args.stream {
                    for item in items.items {
                        sender.send(item.into()).await?;
                    }
                } else {
                    sender.send(serde_json::to_value(items)?.into()).await?;
                }
            }
            #[cfg(not(feature = "duckdb"))]
            {
                if args.duckdb == Some(true) {
                    return Err(crate::Error::Custom(format!("`--duckdb true` was provided, but this crate was not compiled with duckdb support")));
                } else {
                    return Err(crate::Error::Custom(format!("The search href was auto-detected as geoparquet, but this crate was not compiled with DuckDB support. Either re-install with `-F duckdb` or disable geoparquet auto-detection with `--duckdb false`")));
                }
            }
        } else {
            let client = ApiClient::new(&args.href)?;
            let item_stream = client.search(search).await?;
            tokio::pin!(item_stream);
            let mut num_items = 0;
            if args.stream {
                while let Some(result) = item_stream.next().await {
                    let item: Item = result?;
                    num_items += 1;
                    sender.send(item.into()).await?;
                    if args
                        .max_items
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
                    if args
                        .max_items
                        .map(|max_items| num_items >= max_items)
                        .unwrap_or(false)
                    {
                        break;
                    }
                }
                let item_collection = ItemCollection::new(items)?;
                sender
                    .send(serde_json::to_value(item_collection)?.into())
                    .await?;
            }
        }
        Ok(())
    }
}
