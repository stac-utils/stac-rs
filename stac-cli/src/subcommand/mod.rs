mod item;
mod search;
mod serve;
mod sort;
mod translate;
mod validate;

use crate::{
    Error, Format, ItemArgs, Output, Result, SearchArgs, ServeArgs, SortArgs, TranslateArgs,
    ValidateArgs,
};
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

    /// Translates STAC values between formats.
    Translate(TranslateArgs),

    /// Validates a STAC object or API endpoint using json-schema validation.
    Validate(ValidateArgs),
}

impl Subcommand {
    pub(crate) fn infile(&self) -> Option<&str> {
        use Subcommand::*;

        match self {
            Item(args) => Some(args.id_or_href.as_str()),
            Sort(args) => args.infile.as_deref(),
            Translate(args) => args.infile.as_deref(),
            Validate(args) => args.href.as_deref(),
            _ => None,
        }
    }

    pub(crate) fn outfile(&self) -> Option<&str> {
        use Subcommand::*;

        match self {
            Item(args) => args.outfile.as_deref(),
            Search(args) => args.outfile.as_deref(),
            Sort(args) => args.outfile.as_deref(),
            Translate(args) => args.outfile.as_deref(),
            _ => None,
        }
    }

    pub(crate) async fn run(self, input_format: Format, sender: Sender<Output>) -> Result<()> {
        use Subcommand::*;

        match self {
            Item(args) => {
                let item = Subcommand::item(args)?;
                sender.send(item.into()).await?;
            }
            Search(args) => Subcommand::search(args, sender).await?,
            Serve(args) => Subcommand::serve(args, sender).await?,
            Sort(args) => {
                let value = Subcommand::sort(args, input_format).await?;
                sender.send(value.into()).await?;
            }
            Translate(args) => {
                let value = Subcommand::translate(args, input_format).await?;
                sender.send(value.into()).await?;
            }
            Validate(args) => {
                if let Err(err) = Subcommand::validate(args, input_format).await {
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
