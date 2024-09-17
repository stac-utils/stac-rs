use super::{Input, Run};
use crate::{Error, Result, Value};
use serde::de::DeserializeOwned;
use stac_api::{Client, GetItems, ItemCollection, Items, Search};
use std::{fs::File, io::BufReader};
use tokio::sync::mpsc::Sender;
use tokio_stream::StreamExt;
use tracing::info;

#[derive(Debug, clap::Args)]
pub(crate) struct Args {
    /// The href of the STAC API or the stac-geoparquet file to search
    #[cfg(feature = "duckdb")]
    href: String,

    /// The href of the STAC API to search
    #[cfg(not(feature = "duckdb"))]
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

    /// `cql2-text` or `cql2-json`. If undefined, defaults to cql2-text for a GET request and cql2-json for a POST request.
    #[arg(long)]
    filter_lang: Option<String>,

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

async fn search_api(
    href: String,
    search: Search,
    sender: Option<Sender<Value>>,
    max_items: Option<usize>,
) -> Result<Option<Value>> {
    info!("searching '{}' using an api client", href);
    let client = Client::new(&href)?;
    let stream = client.search(search).await?;
    tokio::pin!(stream);
    let mut count = 0;
    if let Some(sender) = sender {
        while let Some(item) = stream.next().await {
            let item = item?;
            count += 1;
            sender.send(item.into()).await?;
            if max_items
                .map(|max_items| count >= max_items)
                .unwrap_or_default()
            {
                break;
            }
        }
        Ok(None)
    } else {
        let mut items = if let Some(max_items) = max_items {
            Vec::with_capacity(max_items)
        } else {
            Vec::new()
        };
        while let Some(item) = stream.next().await {
            items.push(item?);
            count += 1;
            if max_items
                .map(|max_items| count >= max_items)
                .unwrap_or_default()
            {
                break;
            }
        }
        let item_collection = ItemCollection::new(items)?;
        Ok(Some(Value::Json(serde_json::to_value(item_collection)?)))
    }
}

#[cfg(feature = "duckdb")]
async fn search_geoparquet(
    href: String,
    mut search: Search,
    sender: Option<Sender<Value>>,
    max_items: Option<usize>,
) -> Result<Option<Value>> {
    info!("searching '{}' using duckdb", href);
    if let Some(max_items) = max_items {
        let max_items: u64 = max_items.try_into()?;
        if search.limit.map(|limit| limit > max_items).unwrap_or(true) {
            search.limit = Some(max_items);
        }
    }
    let client = stac_duckdb::Client::from_href(href)?;
    let items = client.search_to_json(search)?;
    if let Some(sender) = sender {
        for item in items.items {
            sender.send(item.into()).await?;
        }
        Ok(None)
    } else {
        Ok(Some(Value::Json(serde_json::to_value(items)?)))
    }
}

impl Run for Args {
    async fn run(self, _: Input, stream: Option<Sender<Value>>) -> Result<Option<Value>> {
        let get_items = GetItems {
            bbox: self.bbox,
            datetime: self.datetime,
            fields: self.fields,
            sortby: self.sortby,
            filter_crs: self.filter_crs,
            filter_lang: self.filter_lang,
            filter: self.filter,
            ..Default::default()
        };
        let mut items: Items = get_items.try_into()?;
        items.limit = self.limit;
        items.query = self.query.map(json).transpose()?;
        let mut search: Search = items.into();
        search.intersects = self.intersects.map(json).transpose()?;
        search.ids = if self.ids.is_empty() {
            None
        } else {
            Some(self.ids)
        };
        search.collections = if self.collections.is_empty() {
            None
        } else {
            Some(self.collections)
        };
        #[cfg(feature = "duckdb")]
        {
            if self.duckdb.unwrap_or_else(|| {
                matches!(
                    stac::Format::infer_from_href(&self.href),
                    Some(stac::Format::Geoparquet(_))
                )
            }) {
                search_geoparquet(self.href, search, stream, self.max_items).await
            } else {
                search_api(self.href, search, stream, self.max_items).await
            }
        }
        #[cfg(not(feature = "duckdb"))]
        {
            search_api(self.href, search, stream, self.max_items).await
        }
    }

    fn take_outfile(&mut self) -> Option<String> {
        self.outfile.take()
    }
}

fn json<T>(s: String) -> Result<T>
where
    T: DeserializeOwned,
{
    if s.starts_with('@') && s.len() > 1 {
        info!("reading {}", &s[1..]);
        let file = BufReader::new(File::open(&s[1..])?);
        serde_json::from_reader(file).map_err(Error::from)
    } else {
        serde_json::from_str(&s).map_err(Error::from)
    }
}
