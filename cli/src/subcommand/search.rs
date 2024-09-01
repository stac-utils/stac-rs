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
        Ok(())
    }
}
