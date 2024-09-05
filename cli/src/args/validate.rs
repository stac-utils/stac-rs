use super::{Input, Run};
use crate::{Error, Result, Value};
use stac_validate::Validate;
use tokio::sync::mpsc::Sender;

/// Arguments for the `validate` subcommand.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The input file, if not provided or `-` the input will be read from standard input
    infile: Option<String>,

    /// The output file, if not provided the item will be printed to standard output
    outfile: Option<String>,
}

impl Run for Args {
    async fn run(self, input: Input, sender: Sender<Value>) -> Result<Option<Value>> {
        let value = input.read(self.infile)?;
        let result = value.validate();
        if let Err(stac_validate::Error::Validation(ref errors)) = result {
            let message_base = match value {
                stac::Value::Item(item) => format!("[item={}] ", item.id),
                stac::Value::Catalog(catalog) => format!("[catalog={}] ", catalog.id),
                stac::Value::Collection(collection) => format!("[collection={}] ", collection.id),
                stac::Value::ItemCollection(_) => "[item-collection] ".to_string(),
            };
            for error in errors {
                let message = format!(
                    "{}{} (instance path: '{}', schema path: '{}')",
                    message_base, error, error.instance_path, error.schema_path
                );
                sender.send(message.into()).await?;
            }
        }
        result.and(Ok(None)).map_err(Error::from)
    }

    fn take_outfile(&mut self) -> Option<String> {
        self.outfile.take()
    }
}
