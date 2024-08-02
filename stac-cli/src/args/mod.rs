mod item;
mod search;
mod serve;
mod sort;
mod validate;

use crate::{Printer, Subcommand};
use clap::Parser;
pub use {
    item::ItemArgs, search::SearchArgs, serve::ServeArgs, sort::SortArgs, validate::ValidateArgs,
};

/// CLI arguments.
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// Use a compact representation of the output, if possible.
    #[arg(short, long)]
    compact: bool,

    /// The subcommand to run.
    #[command(subcommand)]
    pub subcommand: Subcommand,
}

impl Args {
    /// Executes the subcommand.
    pub async fn execute(self) -> i32 {
        use Subcommand::*;
        let printer = Printer::new(self.compact);
        let result = match self.subcommand {
            Item(item_args) => item_args.execute(printer),
            Search(search_args) => search_args.execute(printer).await,
            Serve(serve_args) => serve_args.execute(printer).await,
            Sort(sort_args) => sort_args.execute(printer).await,
            Validate(validate_args) => validate_args.execute(printer).await,
        };
        match result {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("ERROR: {}", err);
                err.code()
            }
        }
    }
}
