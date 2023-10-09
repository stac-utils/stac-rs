use crate::Result;
use clap::Subcommand;
use stac_validate::Validate;

#[derive(Debug, Subcommand)]
pub enum Command {
    /// Sorts the fields of STAC object.
    Sort {
        /// The href of the STAC object.
        href: String,

        /// If true, don't pretty-print the output
        #[arg(short, long)]
        compact: bool,
    },

    /// Validates a STAC object using json-schema validation.
    Validate {
        /// The href of the STAC object.
        href: String,
    },
}

impl Command {
    pub async fn execute(self) -> Result<()> {
        use Command::*;
        match self {
            Sort { href, compact } => sort(&href, compact).await,
            Validate { href } => validate(&href).await,
        }
    }
}

async fn validate(href: &str) -> Result<()> {
    let value: serde_json::Value = stac_async::read_json(href).await?;
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

async fn sort(href: &str, compact: bool) -> Result<()> {
    let value: stac::Value = stac_async::read_json(href).await?;
    let output = if compact {
        serde_json::to_string(&value).unwrap()
    } else {
        serde_json::to_string_pretty(&value).unwrap()
    };
    println!("{}", output);
    Ok(())
}
