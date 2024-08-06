mod item;
mod search;
mod serve;
mod sort;
mod validate;

use crate::{Error, ItemArgs, Output, Result, SearchArgs, ServeArgs, SortArgs, ValidateArgs};
use tokio::sync::mpsc::Sender;

/// A CLI subcommand.
#[derive(Debug, clap::Subcommand)]
#[allow(clippy::large_enum_variant)]
pub enum Subcommand {
    /// Creates a STAC Item.
    Item(ItemArgs),

    /// Searches a STAC API.
    Search(SearchArgs),

    /// Serves a STAC API.
    ///
    /// By default, uses a basic memory backend, which is not suitable for
    /// production. To use the pgstac backend, pass the pgstac connection string
    /// to the `--pgstac` argument.
    Serve(ServeArgs),

    /// Sorts the fields of STAC object.
    Sort(SortArgs),

    /// Validates a STAC object or API endpoint using json-schema validation.
    Validate(ValidateArgs),
}

impl Subcommand {
    pub(crate) async fn run(self, sender: Sender<Output>) -> Result<()> {
        use Subcommand::*;

        match self {
            Item(args) => {
                let item = Subcommand::item(args)?;
                sender.send(item.into()).await?;
            }
            Search(args) => Subcommand::search(args, sender).await?,
            Serve(args) => Subcommand::serve(args, sender).await?,
            Sort(args) => {
                let value = Subcommand::sort(args).await?;
                sender.send(value.into()).await?;
            }
            Validate(args) => {
                if let Err(err) = Subcommand::validate(args).await {
                    match err {
                        Error::Validation(errors) => {
                            for error in errors {
                                sender.send(error.into()).await?;
                            }
                            sender
                                .send("one or more errors during validation".into())
                                .await?;
                        }
                        _ => return Err(err),
                    }
                }
            }
        }
        Ok(())
    }
}
