use super::{Input, Run};
use crate::{Result, Value};
use tokio::sync::mpsc::Sender;

/// Arguments for the `translate` subcommand.
#[derive(clap::Args, Debug)]
pub(crate) struct Args {
    /// The input file, if not provided or `-` the input will be read from standard input
    infile: Option<String>,

    /// The output file, if not provided the item will be printed to standard output
    outfile: Option<String>,
}

impl Run for Args {
    async fn run(self, input: Input, _: Option<Sender<Value>>) -> Result<Option<Value>> {
        input.get().await.map(|value| Some(value.into()))
    }

    fn take_infile(&mut self) -> Option<String> {
        self.infile.take()
    }

    fn take_outfile(&mut self) -> Option<String> {
        self.outfile.take()
    }
}
