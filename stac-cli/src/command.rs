use crate::Result;
use clap::Subcommand;
use stac_api::GetSearch;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Searches a STAC API.
    Search {
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

        /// Array of Item ids to return.
        #[arg(short, long)]
        ids: Option<Vec<String>>,

        /// Array of one or more Collection IDs that each matching Item must be in.
        #[arg(short, long)]
        collections: Option<Vec<String>>,

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

        /// Do not pretty print the output.
        ///
        /// Only used if stream is false.
        #[arg(long)]
        compact: bool,
    },

    /// Sorts the fields of STAC object.
    Sort {
        /// The href of the STAC object.
        href: String,

        /// If true, don't pretty-print the output
        #[arg(short, long)]
        compact: bool,
    },

    /// Validates a STAC object using json-schema validation.
    Validate {
        /// The href of the STAC object.
        href: String,
    },
}

impl Command {
    pub async fn execute(self) -> Result<()> {
        use Command::*;
        match self {
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
                compact,
            } => {
                let get_search = GetSearch {
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
                    additional_fields: Default::default(),
                };
                let search = get_search.try_into()?;
                crate::commands::search(&href, search, max_items, stream, !(compact | stream)).await
            }
            Sort { href, compact } => crate::commands::sort(&href, compact).await,
            Validate { href } => crate::commands::validate(&href).await,
        }
    }
}
