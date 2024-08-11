use crate::{Format, MigrateArgs, Result, Subcommand};
use stac::Value;

impl Subcommand {
    /// Sorts a STAC value.
    pub(crate) async fn migrate(args: MigrateArgs, input_format: Format) -> Result<Value> {
        let mut value: Value = input_format.read_href(args.infile.as_deref()).await?;
        value.migrate(args.version)?;
        Ok(value)
    }
}
