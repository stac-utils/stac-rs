use crate::Result;
use clap::{Parser, Subcommand};
use stac_validate::Validate;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Debug, Subcommand)]
pub enum Command {
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
            Validate { href } => {
                let value: serde_json::Value = stac_async::read_json(&href).await?;
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
