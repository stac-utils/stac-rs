use crate::{Error, Format, Result, Subcommand, ValidateArgs};
use serde_json::json;
use stac_validate::Validate;

impl Subcommand {
    /// Validates a STAC value.
    pub async fn validate(args: ValidateArgs, input_format: Format) -> Result<()> {
        let value: serde_json::Value = input_format.read_href(args.href.as_deref()).await?;
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
            Err(Error::Validation(errors))
        }
    }
}
