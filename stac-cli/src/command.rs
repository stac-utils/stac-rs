use crate::Result;
use clap::Subcommand;
use stac_api::GetSearch;
use std::path::Path;
use url::Url;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Creates a STAC Item from an asset href.
    Item {
        /// The asset href.
        href: String,

        /// The item id.
        ///
        /// If not provided, will be inferred from the filename in the href.
        #[arg(short, long)]
        id: Option<String>,

        /// The asset key.
        #[arg(short, long, default_value = "data")]
        key: String,

        /// The asset roles.
        ///
        /// Can be provided multiple times.
        #[arg(short, long)]
        role: Vec<String>,

        /// Allow relative paths.
        ///
        /// If false, paths will be canonicalized, which requires that the files actually exist on the filesystem.
        #[arg(long)]
        allow_relative_paths: bool,

        /// Use compact representation for the output.
        #[arg(short, long)]
        compact: bool,

        /// Don't use GDAL for item creation.
        ///
        /// Automatically set to true if this crate is compiled without GDAL.
        #[arg(long)]
        disable_gdal: bool,
    },

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
        ///
        /// If this is not provided, will read from standard input.
        href: Option<String>,

        /// If true, don't pretty-print the output
        #[arg(short, long)]
        compact: bool,
    },

    /// Validates a STAC object or API endpoint using json-schema validation.
    Validate {
        /// The href of the STAC object or endpoint.
        ///
        /// The validator will make some decisions depending on what type of
        /// data is returned from the href. If it's a STAC Catalog, Collection,
        /// or Item, that object will be validated. If its a collections
        /// endpoint from a STAC API, all collections will be validated.
        /// Additional behavior TBD.
        ///
        /// If this is not provided, will read from standard input.
        href: Option<String>,
    },
}

impl Command {
    pub async fn execute(self) -> Result<()> {
        use Command::*;
        match self {
            Item {
                id,
                href,
                key,
                role,
                allow_relative_paths,
                compact,
                mut disable_gdal,
            } => {
                let id = id.unwrap_or_else(|| infer_id(&href));
                if !cfg!(feature = "gdal") {
                    disable_gdal = true;
                }
                crate::commands::item(
                    id,
                    href,
                    key,
                    role,
                    allow_relative_paths,
                    compact,
                    disable_gdal,
                )
            }
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
            Sort { href, compact } => crate::commands::sort(href.as_deref(), compact).await,
            Validate { href } => crate::commands::validate(href.as_deref()).await,
        }
    }
}

fn infer_id(href: &str) -> String {
    if let Ok(url) = Url::parse(href) {
        url.path_segments()
            .and_then(|path_segments| path_segments.last())
            .and_then(|path_segment| Path::new(path_segment).file_stem())
            .map(|file_stem| file_stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| href.to_string())
    } else {
        Path::new(href)
            .file_stem()
            .map(|file_stem| file_stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| href.to_string())
    }
}
