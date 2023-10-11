use crate::Result;
use stac_api::{Item, ItemCollection, Search};
use stac_async::ApiClient;
use tokio_stream::StreamExt;

pub async fn search(
    href: &str,
    search: Search,
    max_items: Option<usize>,
    stream: bool,
    pretty: bool,
) -> Result<()> {
    let client = ApiClient::new(href)?;
    let item_stream = client.search(search).await?;
    tokio::pin!(item_stream);
    let mut num_items = 0;
    if stream {
        assert!(!pretty);
        while let Some(result) = item_stream.next().await {
            let item: Item = result?;
            num_items += 1;
            println!("{}", serde_json::to_string(&item)?);
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
        let output = if pretty {
            serde_json::to_string_pretty(&item_collection)?
        } else {
            serde_json::to_string(&item_collection)?
        };
        println!("{}", output);
    }
    Ok(())
}
