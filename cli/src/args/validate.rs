use super::{Input, Run};
use crate::{Error, Result, Value};
use stac_validate::Validate;
use tokio::sync::mpsc::Sender;

/// Arguments for the `validate` subcommand.
#[derive(clap::Args, Debug)]
pub(crate) struct Args {
    /// The input file, if not provided or `-` the input will be read from standard input
    infile: Option<String>,

    /// The output file, if not provided the item will be printed to standard output
    outfile: Option<String>,
}

impl Run for Args {
    async fn run(self, input: Input, _: Option<Sender<Value>>) -> Result<Option<Value>> {
        let value = input.get().await?;
        let result = value.validate().await;
        if let Err(stac_validate::Error::Validation(ref errors)) = result {
            let message_base = match value {
                stac::Value::Item(item) => format!("[item={}] ", item.id),
                stac::Value::Catalog(catalog) => format!("[catalog={}] ", catalog.id),
                stac::Value::Collection(collection) => format!("[collection={}] ", collection.id),
                stac::Value::ItemCollection(_) => "[item-collection] ".to_string(),
            };
            for error in errors {
                eprintln!(
                    "{}{} (instance path: '{}', schema path: '{}')",
                    message_base, error, error.instance_path, error.schema_path
                );
            }
        }
        result.and(Ok(None)).map_err(Error::from)
    }

    fn take_infile(&mut self) -> Option<String> {
        self.infile.take()
    }

    fn take_outfile(&mut self) -> Option<String> {
        self.outfile.take()
    }
}
