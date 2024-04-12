#[derive(Debug, clap::Subcommand)]
pub enum Subcommand {
    /// Creates a STAC Item.
    Item {
        /// The item id or asset href.
        id_or_href: String,

        /// The item id, if the positional argument is an href.
        ///
        /// If not provided, will be inferred from the filename in the href.
        #[arg(short, long)]
        id: Option<String>,

        /// The asset key, if the positional argument is an href.
        #[arg(short, long, default_value = "data")]
        key: String,

        /// The asset roles, if the positional argument is an href.
        ///
        /// Can be provided multiple times.
        #[arg(short, long)]
        role: Vec<String>,

        /// Allow relative paths.
        ///
        /// If false, all path will be canonicalized, which requires that the
        /// files actually exist on the filesystem.
        #[arg(long)]
        allow_relative_paths: bool,

        /// Don't use GDAL for item creation, if the positional argument is an href.
        ///
        /// Automatically set to true if this crate is compiled without GDAL.
        #[arg(long)]
        disable_gdal: bool,

        /// Collect an item or item collection from standard input, and add the
        /// newly created to it into a new item collection.
        #[arg(short, long)]
        collect: bool,
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
    },

    /// Serves a STAC API.
    ///
    /// By default, uses a basic memory backend, which is not suitable for
    /// production. To use the pgstac backend, pass the pgstac connection string
    /// to the `--pgstac` argument.
    Serve {
        /// Hrefs of STAC collections and items to load before starting the server.
        href: Vec<String>,

        /// The pgstac connection string.
        #[arg(long)]
        pgstac: Option<String>,
    },

    /// Sorts the fields of STAC object.
    Sort {
        /// The href of the STAC object.
        ///
        /// If this is not provided, will read from standard input.
        href: Option<String>,
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
