use super::{Input, Run};
use crate::{Error, Result, Value};
use stac::Validate;
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
        let value = input.get_json().await?;
        let result = value.validate().await;
        if let Err(stac::Error::Validation(ref errors)) = result {
            let id = value
                .get("id")
                .and_then(|v| v.as_str())
                .map(|v| format!("={}", v))
                .unwrap_or_default();
            let message_base = match value
                .get("type")
                .and_then(|v| v.as_str())
                .unwrap_or_default()
            {
                "Feature" => format!("[item={}] ", id),
                "Catalog" => format!("[catalog={}] ", id),
                "Collection" => format!("[collection={}] ", id),
                "FeatureCollection" => "[item-collection] ".to_string(),
                _ => String::new(),
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
