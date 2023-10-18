use crate::{Error, Result};
use stac_validate::{Validate, Validator};

pub async fn validate(href: &str) -> Result<()> {
    let value: serde_json::Value = stac_async::read_json(href).await?;
    if let Some(map) = value.as_object() {
        if map.contains_key("type") {
            let value = value.clone();
            let result = tokio::task::spawn_blocking(move || value.validate()).await?;
            print_result(result).map_err(Error::from)
        } else if let Some(collections) = map
            .get("collections")
            .and_then(|collections| collections.as_array())
        {
            let collections = collections.clone();
            let result = tokio::task::spawn_blocking(move || {
                let mut errors = Vec::new();
                let mut validator = Validator::new();
                let num_collections = collections.len();
                let mut valid_collections = 0;
                for collection in collections {
                    if let Some(id) = collection.get("id").and_then(|id| id.as_str()) {
                        println!("== Validating {}", id);
                    }
                    let result = validator.validate(collection);
                    match print_result(result) {
                        Ok(()) => valid_collections += 1,
                        Err(err) => errors.push(err),
                    }
                    println!("")
                }
                println!(
                    "{}/{} collections are valid",
                    valid_collections, num_collections
                );
                if errors.is_empty() {
                    Ok(())
                } else {
                    Err(Error::ValidationGroup(errors))
                }
            })
            .await?;
            result
        } else {
            todo!()
        }
    } else {
        todo!()
    }
}

pub fn print_result(result: stac_validate::Result<()>) -> stac_validate::Result<()> {
    match result {
        Ok(()) => {
            println!("OK!");
            Ok(())
        }
        Err(stac_validate::Error::Validation(errors)) => {
            for err in &errors {
                println!("Validation error at {}: {}", err.instance_path, err)
            }
            Err(stac_validate::Error::Validation(errors))
        }
        Err(err) => {
            println!("Error while validating: {}", err);
            Err(err)
        }
    }
}
