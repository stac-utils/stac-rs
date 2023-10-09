use crate::Command;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    /// The subcommand to run.
    #[command(subcommand)]
    pub command: Command,
}
