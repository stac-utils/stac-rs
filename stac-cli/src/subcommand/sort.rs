use crate::{Result, SortArgs, Subcommand};
use stac::Value;

impl Subcommand {
    /// Sorts a STAC value.
    pub(crate) async fn sort(args: SortArgs) -> Result<Value> {
        crate::io::read_href(args.href.as_deref()).await
    }
}
