use crate::{Format, Result, Subcommand};
use clap::Parser;
use stac::{Version, STAC_VERSION};
use std::{fs::File, io::Write};

/// CLI arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Use a compact representation of the output, if possible.
    #[arg(short, long)]
    pub compact: bool,

    /// The input format. If not provided, the format will be detected from the input file extension when possible.
    #[arg(short, long)]
    pub input_format: Option<Format>,

    /// The output format. If not provided, the format will be detected from the output file extension when possible.
    #[arg(short, long, value_enum)]
    pub output_format: Option<Format>,

    /// The subcommand to run.
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

/// Arguments for creating an item.
#[derive(clap::Args, Debug)]
pub struct ItemArgs {
    /// The item id or asset href.
    pub id_or_href: String,

    /// The item id, if the positional argument is an href.
    ///
    /// If not provided, will be inferred from the filename in the href.
    #[arg(short, long)]
    pub id: Option<String>,

    /// The asset key, if the positional argument is an href.
    #[arg(short, long, default_value = "data")]
    pub key: String,

    /// The asset roles, if the positional argument is an href.
    ///
    /// Can be provided multiple times.
    #[arg(short, long)]
    pub role: Vec<String>,

    /// Allow relative paths.
    ///
    /// If false, all path will be canonicalized, which requires that the
    /// files actually exist on the filesystem.
    #[arg(long)]
    pub allow_relative_paths: bool,

    /// Don't use GDAL for item creation, if the positional argument is an href.
    ///
    /// Automatically set to true if this crate is compiled without GDAL.
    #[arg(long)]
    pub disable_gdal: bool,

    /// Collect an item or item collection from standard input, and add the
    /// newly created to it into a new item collection.
    #[arg(short, long)]
    pub collect: bool,

    /// The file to write the item to.
    ///
    /// If not provided, the item will be written to standard output.
    pub outfile: Option<String>,
}

/// Arguments for searching a STAC API.
#[derive(Debug, clap::Args)]
pub struct SearchArgs {
    /// The href of the STAC API.
    pub href: String,

    /// The maximum number of items to print.
    #[arg(short, long)]
    pub max_items: Option<usize>,

    /// The maximum number of results to return (page size).
    #[arg(short, long)]
    pub limit: Option<String>,

    /// Requested bounding box.
    #[arg(short, long)]
    pub bbox: Option<String>,

    /// Requested bounding box.
    ///
    /// Use double dots `..` for open date ranges.
    #[arg(short, long)]
    pub datetime: Option<String>,

    /// Searches items by performing intersection between their geometry and
    /// provided GeoJSON geometry.
    ///
    /// All GeoJSON geometry types must be supported.
    #[arg(long)]
    pub intersects: Option<String>,

    /// Comma-delimited list of one ore more Item ids to return.
    #[arg(short, long)]
    pub ids: Option<String>,

    /// Comma-delimited list of one or more Collection IDs that each matching
    /// Item must be in.
    #[arg(short, long)]
    pub collections: Option<String>,

    /// Include/exclude fields from item collections.
    #[arg(long)]
    pub fields: Option<String>,

    /// Fields by which to sort results.
    #[arg(short, long)]
    pub sortby: Option<String>,

    /// Recommended to not be passed, but server must only accept
    /// <http://www.opengis.net/def/crs/OGC/1.3/CRS84> as a valid value, may
    /// reject any others
    #[arg(long)]
    pub filter_crs: Option<String>,

    /// CQL2 filter expression.
    #[arg(long)]
    pub filter_lang: Option<String>,

    /// CQL2 filter expression.
    #[arg(short, long)]
    pub filter: Option<String>,

    /// Stream the items to output as ndjson.
    #[arg(long)]
    pub stream: bool,

    /// Search with DuckDB instead of a STAC API client.
    ///
    /// ⚠ Experimental: this is a new feature whose behavior might change without warning.
    ///
    /// If true, or if not provided and the href's extension is `parquet` or `geoparquet`, use DuckDB. Set to `false` to always use a STAC API client.
    #[arg(long)]
    pub duckdb: Option<bool>,

    /// The file to write the output to.
    ///
    /// If not provided, the output will be written to standard output.
    pub outfile: Option<String>,
}

/// Arguments for serving a STAC API.
#[derive(clap::Args, Debug)]
pub struct ServeArgs {
    /// Hrefs of STAC collections and items to load before starting the server.
    pub href: Vec<String>,

    /// The pgstac connection string.
    #[arg(long)]
    pub pgstac: Option<String>,

    /// Don't auto-create collections for items that are missing them.
    #[arg(long)]
    pub dont_auto_create_collections: bool,

    /// Don't follow links in collections to more items.
    #[arg(long)]
    pub dont_follow_links: bool,
}

/// Arguments for sorting a STAC value.
#[derive(clap::Args, Debug)]
pub struct SortArgs {
    /// The href of the STAC to sort.
    ///
    /// If this is not provided, or is `-`, will read from standard input.
    pub infile: Option<String>,

    /// The output filename.
    ///
    /// If this is not provided, output will be printed to standard output.
    pub outfile: Option<String>,
}

/// Arguments for validating a STAC value.
#[derive(clap::Args, Debug)]
pub struct ValidateArgs {
    /// The href of the STAC object or endpoint.
    ///
    /// The validator will make some decisions depending on what type of
    /// data is returned from the href. If it's a STAC Catalog, Collection,
    /// or Item, that object will be validated. If its a collections
    /// endpoint from a STAC API, all collections will be validated.
    /// Additional behavior TBD.
    ///
    /// If this is not provided, or is `-`, will read from standard input.
    pub href: Option<String>,
}

/// Arguments for translating STAC values.
#[derive(clap::Args, Debug)]
pub struct TranslateArgs {
    /// The input STAC value.
    ///
    /// If this is not provided, or is `-`, input will be read from standard
    /// input.
    pub infile: Option<String>,

    /// The output STAC value.
    ///
    /// If not provided, output will be printed to standard output.
    pub outfile: Option<String>,
}

/// Arguments for migrating STAC values.
#[derive(clap::Args, Debug)]
pub struct MigrateArgs {
    /// The input STAC value.
    ///
    /// If this is not provided, or is `-`, input will be read from standard
    /// input.
    pub infile: Option<String>,

    /// The output STAC value.
    ///
    /// If not provided, output will be printed to standard output.
    pub outfile: Option<String>,

    /// The STAC version to migrate to.
    #[arg(short, long, default_value_t = STAC_VERSION)]
    pub version: Version,
}

impl Args {
    pub(crate) fn writer(&self) -> Result<Box<dyn Write + Send>> {
        if let Some(outfile) = self.subcommand.outfile() {
            let file = File::create(outfile)?;
            Ok(Box::new(file))
        } else {
            Ok(Box::new(std::io::stdout()))
        }
    }

    pub(crate) fn input_format(&self) -> Format {
        self.input_format
            .or_else(|| self.subcommand.infile().and_then(Format::maybe_from_href))
            .unwrap_or_default()
    }

    pub(crate) fn output_format(&self) -> Format {
        self.output_format
            .or_else(|| self.subcommand.outfile().and_then(Format::maybe_from_href))
            .unwrap_or_default()
    }

    pub(crate) fn outfile(&self) -> Option<&str> {
        self.subcommand.outfile()
    }
}
