use super::{Input, Run};
use crate::{Error, Result, Value};
use stac::{Migrate, Version, STAC_VERSION};
use tokio::sync::mpsc::Sender;

/// Arguments for the `migrate` subcommand.
#[derive(clap::Args, Debug)]
pub(crate) struct Args {
    /// The input file, if not provided or `-` the input will be read from standard input
    infile: Option<String>,

    /// The output file, if not provided the item will be printed to standard output
    outfile: Option<String>,

    /// The version to migrate to
    #[arg(long, default_value_t = STAC_VERSION)]
    version: Version,
}

impl Run for Args {
    async fn run(self, input: Input, _: Option<Sender<Value>>) -> Result<Option<Value>> {
        let value = input.get().await?;
        value
            .migrate(&self.version)
            .map(Value::from)
            .map(Some)
            .map_err(Error::from)
    }

    fn take_infile(&mut self) -> Option<String> {
        self.infile.take()
    }

    fn take_outfile(&mut self) -> Option<String> {
        self.outfile.take()
    }
}
