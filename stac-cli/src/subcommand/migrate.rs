use crate::{Error, Format, MigrateArgs, Result, Subcommand};
use stac::{Migrate, Value};

impl Subcommand {
    /// Migrates a STAC value.
    pub(crate) async fn migrate(args: MigrateArgs, input_format: Format) -> Result<Value> {
        let value: Value = input_format.read_href(args.infile.as_deref()).await?;
        value.migrate(args.version).map_err(Error::from)
    }
}
