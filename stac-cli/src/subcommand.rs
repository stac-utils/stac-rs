use crate::{ItemArgs, SearchArgs, ServeArgs, SortArgs, ValidateArgs};

/// A CLI subcommand.
#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    /// Creates a STAC Item.
    Item(ItemArgs),

    /// Searches a STAC API.
    Search(SearchArgs),

    /// Serves a STAC API.
    ///
    /// By default, uses a basic memory backend, which is not suitable for
    /// production. To use the pgstac backend, pass the pgstac connection string
    /// to the `--pgstac` argument.
    Serve(ServeArgs),

    /// Sorts the fields of STAC object.
    Sort(SortArgs),

    /// Validates a STAC object or API endpoint using json-schema validation.
    Validate(ValidateArgs),
}
