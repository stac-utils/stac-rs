use crate::{Error, Printer, Result};
use clap::Args;
use serde_json::json;
use stac_validate::Validate;

/// Arguments for validating a STAC value.
#[derive(Args, Debug)]
pub struct ValidateArgs {
    /// The href of the STAC object or endpoint.
    ///
    /// The validator will make some decisions depending on what type of
    /// data is returned from the href. If it's a STAC Catalog, Collection,
    /// or Item, that object will be validated. If its a collections
    /// endpoint from a STAC API, all collections will be validated.
    /// Additional behavior TBD.
    ///
    /// If this is not provided, will read from standard input.
    href: Option<String>,
}

impl ValidateArgs {
    /// Validates a STAC value.
    pub async fn execute(&self, printer: Printer) -> Result<()> {
        let value: serde_json::Value = crate::io::read_href(self.href.as_deref()).await?;
        let mut errors: Vec<serde_json::Value> = Vec::new();
        let mut update_errors = |result: std::result::Result<(), stac_validate::Error>| match result
        {
            Ok(()) => {}
            Err(err) => match err {
                stac_validate::Error::Validation(ref e) => {
                    errors.extend(e.iter().map(|error| {
                        json!({
                                "type": "validation",
                                "instance_path": error.instance_path,
                                "schema_path": error.schema_path,
                                "description": error.to_string(),
                        })
                    }));
                }
                _ => errors.push(json!({
                    "type": "other",
                    "message": err.to_string(),
                })),
            },
        };
        if let Some(collections) = value
            .as_object()
            .and_then(|object| object.get("collections"))
        {
            if let Some(collections) = collections.as_array() {
                for collection in collections.iter() {
                    let collection = collection.clone();
                    let result = tokio::task::spawn_blocking(move || collection.validate()).await?;
                    update_errors(result);
                }
            } else {
                return Err(Error::Custom(
                    "expected the 'collections' key to be an array".to_string(),
                ));
            }
        } else {
            let result = tokio::task::spawn_blocking(move || value.validate()).await?;
            update_errors(result);
        }
        if errors.is_empty() {
            Ok(())
        } else {
            printer.println(errors)?;
            Err(Error::Custom(
                "one or more errors during validation".to_string(),
            ))
        }
    }
}
