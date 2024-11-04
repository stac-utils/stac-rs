use crate::Result;
use stac::{Migrate, Version, STAC_VERSION};

/// Arguments for the `translate` subcommand.
#[derive(clap::Args, Debug, Clone)]
pub struct Args {
    /// The input file.
    ///
    /// If not provided or `-`, the input will be read from standard input.
    infile: Option<String>,

    /// The output file.
    ///
    /// If not provided or `-` the item will be printed to standard output.
    outfile: Option<String>,

    /// The output version.
    #[arg(long, default_value_t = STAC_VERSION)]
    version: Version,
}

impl crate::Args {
    pub async fn translate(&self, args: &Args) -> Result<()> {
        let value = self.get(args.infile.clone()).await?;
        let value = value.migrate(&args.version)?;
        self.put(value, args.outfile.as_deref()).await
    }
}
