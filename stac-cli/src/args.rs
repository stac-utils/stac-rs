use crate::{Error, Result};
use clap::{Parser, Subcommand};
use stac::{Validate, Value};
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
                match stac_async::read(href).await? {
                    Collection(collection) => {
                        crate::download(collection, directory, create_directory).await?
                    }
                    Item(item) => crate::download(item, directory, create_directory).await?,
                    _ => unimplemented!(),
                }
                Ok(())
            }
            Validate { href } => {
                let value: Value = stac_async::read(href).await?;
                let result = {
                    let value = value.clone();
                    // TODO when https://github.com/gadomski/stac-rs/issues/118
                    // is fixed, switch to using async validation.
                    tokio::task::spawn_blocking(move || value.validate()).await?
                };
                if let Err(err) = result {
                    for err in err {
                        match err {
                            stac::Error::ValidationError(err) => {
                                println!("Validation error at {}: {}", err.instance_path, err)
                            }
                            _ => println!("{}", err),
                        }
                    }
                    Err(Error::InvalidValue(value))
                } else {
                    println!("OK!");
                    Ok(())
                }
            }
        }
    }
}
