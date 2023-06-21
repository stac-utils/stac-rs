use crate::{Error, Result};
use clap::{Parser, Subcommand};
use stac::Value;
use stac_validate::Validate;
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

    /// Validate a STAC object using json-schema validation.
    Validate {
        /// The href of the STAC object.
        href: String,
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
                let value: Value = stac_async::read(href).await?;
                match value {
                    Collection(collection) => {
                        crate::download(collection, directory, create_directory).await
                    }
                    Item(item) => crate::download(item, directory, create_directory).await,
                    _ => Err(Error::CannotDownload(value)),
                }
            }
            Validate { href } => {
                let value: Value = stac_async::read(href).await?;
                let result = {
                    let value = value.clone();
                    tokio::task::spawn_blocking(move || value.validate()).await?
                };
                match result {
                    Ok(()) => {
                        println!("OK!");
                        Ok(())
                    }
                    Err(stac_validate::Error::Validation(errors)) => {
                        for err in &errors {
                            println!("Validation error at {}: {}", err.instance_path, err)
                        }
                        Err(stac_validate::Error::Validation(errors).into())
                    }
                    Err(err) => {
                        println!("Error while validating: {}", err);
                        Err(err.into())
                    }
                }
            }
        }
    }
}
