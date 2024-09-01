use crate::{Format, Result, SortArgs, Subcommand};
use stac::Value;

impl Subcommand {
    /// Sorts a STAC value.
    pub(crate) async fn sort(args: SortArgs, input_format: Format) -> Result<Value> {
        input_format.read_href(args.infile.as_deref()).await
    }
}
