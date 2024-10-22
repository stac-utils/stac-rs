use super::{item::Args as ItemArgs, Input, Run};
use crate::{Result, Value};
use tokio::{sync::mpsc::Sender, task::JoinSet};

/// Arguments for the `items` subcommand.
#[derive(clap::Args, Debug)]
pub(crate) struct Args {
    /// The asset hrefs
    hrefs: Vec<String>,

    /// The output file, if not provided the items will be printed to standard output
    #[arg(long)]
    outfile: Option<String>,

    /// The assets' key
    #[arg(short, long, default_value = "data")]
    key: String,

    /// Roles to use for the created assets
    #[arg(short, long = "role", default_values_t = ["data".to_string()])]
    roles: Vec<String>,

    /// Allow assets to have relative hrefs
    #[arg(long)]
    allow_relative_hrefs: bool,
}

impl Run for Args {
    async fn run(self, input: Input, stream: Option<Sender<Value>>) -> Result<Option<Value>> {
        let mut join_set = JoinSet::new();
        let mut items = Vec::with_capacity(self.hrefs.len());
        for href in self.hrefs {
            let input = input.with_href(href.clone());
            let sender = stream.clone();
            let args = ItemArgs {
                id_or_href: href,
                outfile: None,
                key: self.key.clone(),
                roles: self.roles.clone(),
                allow_relative_hrefs: self.allow_relative_hrefs,
            };
            let _ = join_set.spawn(async move { args.run(input, sender).await });
        }
        while let Some(result) = join_set.join_next().await {
            if let Some(Value::Stac(value)) = result?? {
                if let stac::Value::Item(item) = *value {
                    if let Some(ref stream) = stream {
                        stream.send(stac::Value::Item(item).into()).await?;
                    } else {
                        items.push(item);
                    }
                }
            }
        }
        if stream.is_some() {
            Ok(None)
        } else {
            Ok(Some(stac::Value::ItemCollection(items.into()).into()))
        }
    }

    fn take_outfile(&mut self) -> Option<String> {
        self.outfile.take()
    }
}
