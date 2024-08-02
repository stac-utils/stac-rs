use crate::{Printer, Result};
use clap::Args;
use stac_api::{GetItems, GetSearch, Item, ItemCollection};
use stac_async::ApiClient;
use tokio_stream::StreamExt;

/// Arguments for searching a STAC API.
#[derive(Debug, Args)]
pub struct SearchArgs {
    /// The href of the STAC API.
    href: String,

    /// The maximum number of items to print.
    #[arg(short, long)]
    max_items: Option<usize>,

    /// The maximum number of results to return (page size).
    #[arg(short, long)]
    limit: Option<String>,

    /// Requested bounding box.
    #[arg(short, long)]
    bbox: Option<String>,

    /// Requested bounding box.
    /// Use double dots `..` for open date ranges.
    #[arg(short, long)]
    datetime: Option<String>,

    /// Searches items by performing intersection between their geometry and provided GeoJSON geometry.
    ///
    /// All GeoJSON geometry types must be supported.
    #[arg(long)]
    intersects: Option<String>,

    /// Comma-delimited list of one ore more Item ids to return.
    #[arg(short, long)]
    ids: Option<String>,

    /// Comma-delimited list of one or more Collection IDs that each matching Item must be in.
    #[arg(short, long)]
    collections: Option<String>,

    /// Include/exclude fields from item collections.
    #[arg(long)]
    fields: Option<String>,

    /// Fields by which to sort results.
    #[arg(short, long)]
    sortby: Option<String>,

    /// Recommended to not be passed, but server must only accept
    /// <http://www.opengis.net/def/crs/OGC/1.3/CRS84> as a valid value, may
    /// reject any others
    #[arg(long)]
    filter_crs: Option<String>,

    /// CQL2 filter expression.
    #[arg(long)]
    filter_lang: Option<String>,

    /// CQL2 filter expression.
    #[arg(short, long)]
    filter: Option<String>,

    /// Stream the items to standard output as ndjson.
    #[arg(long)]
    stream: bool,
}

impl SearchArgs {
    /// Search a STAC API.
    pub async fn execute(self, printer: Printer) -> Result<()> {
        let get_items = GetItems {
            limit: self.limit,
            bbox: self.bbox,
            datetime: self.datetime,
            fields: self.fields,
            sortby: self.sortby,
            filter_crs: self.filter_crs,
            filter_lang: self.filter_lang,
            filter: self.filter,
            additional_fields: Default::default(),
        };
        let get_search = GetSearch {
            intersects: self.intersects,
            ids: self.ids,
            collections: self.collections,
            items: get_items,
        };
        let search = get_search.try_into()?;
        let client = ApiClient::new(&self.href)?;
        let item_stream = client.search(search).await?;
        tokio::pin!(item_stream);
        let mut num_items = 0;
        if self.stream {
            while let Some(result) = item_stream.next().await {
                let item: Item = result?;
                num_items += 1;
                printer.println_compact(item)?;
                if self
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
                if self
                    .max_items
                    .map(|max_items| num_items >= max_items)
                    .unwrap_or(false)
                {
                    break;
                }
            }
            let item_collection = ItemCollection::new(items)?;
            printer.println(item_collection)?;
        }
        Ok(())
    }
}
