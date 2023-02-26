use crate::Result;
use clap::{Parser, Subcommand};
use stac::Value;
use std::path::PathBuf;

#[derive(Parser, Debug)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Download assets.
    Download {
        /// The href of the STAC object.
        href: String,

        /// Assets will be downloaded to this directory.
        directory: PathBuf,

        /// If the directory does not exist, should be it be created?
        #[arg(short, long, default_value_t = true)]
        create_directory: bool,
    },
}

impl Command {
    pub async fn execute(self) -> Result<()> {
        use Command::*;
        match self {
            Download {
                href,
                directory,
                create_directory,
            } => {
                use Value::*;
                match stac_async::read(href).await? {
                    Collection(collection) => {
                        crate::download(collection, directory, create_directory).await?
                    }
                    Item(item) => crate::download(item, directory, create_directory).await?,
                    _ => unimplemented!(),
                }
                Ok(())
            }
        }
    }
}
