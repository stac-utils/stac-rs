use crate::{Printer, Result};
use clap::Args;
use serde_json::Value;

/// Arguments for sorting a STAC value.
#[derive(Args, Debug)]
pub struct SortArgs {
    /// The href of the STAC object.
    ///
    /// If this is not provided, will read from standard input.
    href: Option<String>,
}

impl SortArgs {
    /// Sorts a STAC value.
    pub async fn execute(&self, printer: Printer) -> Result<()> {
        let value: Value = crate::io::read_href(self.href.as_deref()).await?;
        printer.println(value)
    }
}
