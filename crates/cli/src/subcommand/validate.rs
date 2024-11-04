use crate::Result;
use stac::Validate;

/// Arguments for the `validate` subcommand.
#[derive(clap::Args, Debug)]
pub struct Args {
    /// The input file.
    ///
    /// If not provided or `-`, the input will be read from standard input.
    infile: Option<String>,
}

impl crate::Args {
    pub async fn validate(&self, args: &Args) -> Result<()> {
        let value = self.get(args.infile.as_deref()).await?;
        if let Err(error) = value.validate().await {
            if let stac::Error::Validation(errors) = &error {
                for error in errors {
                    eprintln!("{}", error);
                }
            }
            Err(error.into())
        } else {
            Ok(())
        }
    }
}
