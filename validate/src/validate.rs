use crate::{Error, Result, Validator};
use serde::Serialize;
use serde_json::Value;
use stac::{Catalog, Collection, Item, ItemCollection};

/// Trait for validating STAC objects using [jsonschema].
pub trait Validate: ValidateCore {
    /// Validate this STAC object using [jsonschema].
    ///
    /// #  Examples
    ///
    /// [stac::Item] implements [Validate]:
    ///
    /// ```
    /// use stac_validate::Validate;
    /// let item = stac::Item::new("an-id");
    /// item.validate().unwrap();
    /// ```
    fn validate(&self) -> Result<()> {
        let mut validator = Validator::new();
        self.validate_with_validator(&mut validator)
    }

    /// Validates a STAC object with the provided validator.
    ///
    /// #  Examples
    ///
    /// [stac::Item] implements [Validate]:
    ///
    /// ```
    /// use stac_validate::{Validate, Validator};
    ///
    /// let mut validator = Validator::new();
    /// let item = stac::Item::new("an-id");
    /// item.validate_with_validator(&mut validator).unwrap();
    /// ```
    fn validate_with_validator(&self, validator: &mut Validator) -> Result<()> {
        let value = serde_json::to_value(self)?;
        let mut errors = match Self::validate_core_json(&value, validator) {
            Ok(()) => Vec::new(),
            Err(Error::Validation(errors)) => errors,
            Err(err) => return Err(err),
        };
        match validator.validate_extensions(&value) {
            Ok(()) => {}
            Err(Error::Validation(extension_errors)) => errors.extend(extension_errors),
            Err(err) => return Err(err),
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Error::Validation(errors))
        }
    }
}

/// Trait for validating STAC objects against their core schema only.
///
/// If your STAC object is the same version as [stac::STAC_VERSION], this will
/// be a quick, cheap operation, since the schemas are stored in the library.
pub trait ValidateCore: Serialize {
    /// Validate a [serde_json::Value] against a specific STAC jsonschema.
    ///
    /// #  Examples
    ///
    /// [stac::Item] implements [ValidateCore]:
    ///
    /// ```
    /// use stac_validate::ValidateCore;
    /// use stac::Item;
    ///
    /// let item = Item::new("an-id");
    /// let value = serde_json::to_value(item).unwrap();
    /// Item::validate_core_json(&value, &mut stac_validate::Validator::new()).unwrap();
    /// ```
    fn validate_core_json(value: &Value, validator: &mut Validator) -> Result<()>;
}

impl Validate for Item {}

impl ValidateCore for Item {
    fn validate_core_json(value: &Value, validator: &mut Validator) -> Result<()> {
        validator.validate_item(value)
    }
}

impl Validate for Catalog {}

impl ValidateCore for Catalog {
    fn validate_core_json(value: &Value, validator: &mut Validator) -> Result<()> {
        validator.validate_catalog(value)
    }
}

impl Validate for Collection {}

impl ValidateCore for Collection {
    fn validate_core_json(value: &Value, validator: &mut Validator) -> Result<()> {
        validator.validate_collection(value)
    }
}

impl Validate for stac::Value {}

impl ValidateCore for stac::Value {
    fn validate_core_json(value: &Value, validator: &mut Validator) -> Result<()> {
        if let Some(type_) = value.get("type") {
            if let Some(type_) = type_.as_str() {
                match type_ {
                    "Feature" => validator.validate_item(value),
                    "Collection" => validator.validate_collection(value),
                    "Catalog" => validator.validate_catalog(value),
                    "FeatureCollection" => validator.validate_item_collection(value),
                    _ => Err(stac::Error::UnknownType(type_.to_string()).into()),
                }
            } else {
                Err(stac::Error::InvalidTypeField(type_.clone()).into())
            }
        } else {
            Err(stac::Error::MissingType.into())
        }
    }
}

impl Validate for ItemCollection {}

impl ValidateCore for ItemCollection {
    fn validate_core_json(value: &Value, validator: &mut Validator) -> Result<()> {
        validator.validate_item_collection(value)
    }
}

impl Validate for Value {}

impl ValidateCore for Value {
    fn validate_core_json(value: &Value, validator: &mut Validator) -> Result<()> {
        if let Some(type_) = value.get("type") {
            if let Some(type_) = type_.as_str() {
                match type_ {
                    "Feature" => validator.validate_item(value),
                    "Collection" => validator.validate_collection(value),
                    "Catalog" => validator.validate_catalog(value),
                    "FeatureCollection" => validator.validate_item_collection(value),
                    _ => Err(stac::Error::UnknownType(type_.to_string()).into()),
                }
            } else {
                Err(stac::Error::InvalidTypeField(type_.clone()).into())
            }
        } else {
            Err(stac::Error::MissingType.into())
        }
    }
}
