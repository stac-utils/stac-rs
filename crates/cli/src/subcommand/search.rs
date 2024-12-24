use crate::{Error, Result};
use serde::de::DeserializeOwned;
use stac_api::{Client, GetItems, ItemCollection, Items, Search};
use std::{fs::File, io::BufReader};
use tokio_stream::StreamExt;

#[derive(Debug, clap::Args, Clone)]
pub struct Args {
    /// The href to search.
    href: String,

    /// The maximum number of items to return
    #[arg(short, long)]
    max_items: Option<usize>,

    /// The maximum number of results to return from the server per page.
    #[arg(short, long)]
    limit: Option<u64>,

    /// Requested bounding box.
    #[arg(short, long)]
    bbox: Option<String>,

    /// Single date+time, or a range ('/' separator), formatted to RFC 3339, section 5.6. Use double dots .. for open date ranges.
    #[arg(short, long)]
    datetime: Option<String>,

    /// Searches items by performing intersection between their geometry and
    /// provided GeoJSON geometry. If prefixed with a `@`, the data will be read from a the file.
    #[arg(long)]
    intersects: Option<String>,

    /// Comma-delimited list of one ore more item ids to return
    #[arg(long, num_args=0.., value_delimiter=',')]
    ids: Vec<String>,

    /// Comma-delimited list of one or more collection IDs that each matching item must be in.
    #[arg(short, long, num_args=0.., value_delimiter=',')]
    collections: Vec<String>,

    /// Include/exclude fields from item collections, e.g. `id,geometry,properties.datetime``
    #[arg(long)]
    fields: Option<String>,

    /// Fields by which to sort results, e.g. `+properties.created`
    #[arg(long)]
    sortby: Option<String>,

    /// Recommended to not be passed, but server must only accept
    /// <http://www.opengis.net/def/crs/OGC/1.3/CRS84> as a valid value, may
    /// reject any others
    #[arg(long)]
    filter_crs: Option<String>,

    /// CQL2 filter expression.
    #[arg(short, long)]
    filter: Option<String>,

    /// A query to pass, if prefixed with a `@` will be read from a file.
    #[arg(long)]
    query: Option<String>,

    /// Whether to search with DuckDB instead of a STAC API client.
    ///
    /// ⚠ Experimental: this is a new feature whose behavior might change without warning.
    ///
    /// If true, or if not provided and the href's extension is `parquet` or `geoparquet`, DuckDB will be used to search. Set to `false` to always use a STAC API client.
    #[arg(long)]
    #[cfg(feature = "duckdb")]
    duckdb: Option<bool>,

    /// The file to write the output to.
    ///
    /// If not provided, the output will be written to standard output.
    outfile: Option<String>,
}

impl crate::Args {
    pub async fn search(&self, args: &Args) -> Result<()> {
        if self.stream && args.outfile.is_some() {
            tracing::warn!("streaming to a file is not supported");
        }
        let get_items = GetItems {
            bbox: args.bbox.clone(),
            datetime: args.datetime.clone(),
            fields: args.fields.clone(),
            sortby: args.sortby.clone(),
            filter_crs: args.filter_crs.clone(),
            filter: args.filter.clone(),
            ..Default::default()
        };
        let mut items: Items = get_items.try_into()?;
        items.limit = args.limit;
        items.query = args.query.as_deref().map(json).transpose()?;
        let mut search: Search = items.into();
        search.intersects = args.intersects.as_deref().map(json).transpose()?;
        search.ids = args.ids.clone();
        search.collections = args.collections.clone();
        #[cfg(feature = "duckdb")]
        let value = if args
            .duckdb
            .or_else(|| Some(stac::Format::is_geoparquet_href(&args.href)))
            .unwrap_or_default()
        {
            self.search_duckdb(args, search).await?
        } else {
            self.search_api(args, search).await?
        };
        #[cfg(not(feature = "duckdb"))]
        let value = self.search_api(args, search).await?;
        if let Some(value) = value {
            let value = serde_json::to_value(value)?;
            self.put(value, args.outfile.as_deref()).await?;
        }
        Ok(())
    }

    async fn search_api(&self, args: &Args, search: Search) -> Result<Option<ItemCollection>> {
        tracing::info!("search {} with a STAC API client", args.href);
        tracing::debug!("search: {:?}", search);
        let client = Client::new(&args.href)?;
        let stream = client.search(search).await?;
        tokio::pin!(stream);
        let mut count = 0;
        let mut items = if !self.stream && args.max_items.is_some() {
            Vec::with_capacity(args.max_items.unwrap())
        } else {
            Vec::new()
        };
        while let Some(result) = stream.next().await {
            let item = result?;
            count += 1;
            if self.stream {
                // We've already warned about the fact that we can't stream to an outfile
                self.put(item, None).await?;
            } else {
                items.push(item);
            }
            if args
                .max_items
                .map(|max_items| count >= max_items)
                .unwrap_or_default()
            {
                break;
            }
        }
        ItemCollection::new(items).map(Some).map_err(Error::from)
    }

    #[cfg(feature = "duckdb")]
    async fn search_duckdb(
        &self,
        args: &Args,
        mut search: Search,
    ) -> Result<Option<ItemCollection>> {
        tracing::info!("search {} with a STAC API client", args.href);
        if let Some(max_items) = args.max_items {
            let max_items: u64 = max_items.try_into()?;
            if search.limit.map(|limit| limit > max_items).unwrap_or(true) {
                search.limit = Some(max_items);
            }
        }
        let client = stac_duckdb::Client::new()?;
        let item_collection = client.search_to_json(&args.href, search)?;
        if self.stream {
            for item in item_collection.items {
                self.put(item, None).await?;
            }
            Ok(None)
        } else {
            Ok(Some(item_collection))
        }
    }
}

fn json<T>(s: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    if s.starts_with('@') && s.len() > 1 {
        tracing::info!("reading {} as JSON", &s[1..]);
        let file = BufReader::new(File::open(&s[1..])?);
        serde_json::from_reader(file).map_err(Error::from)
    } else {
        serde_json::from_str(s).map_err(Error::from)
    }
}
