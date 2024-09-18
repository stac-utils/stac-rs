use crate::{Result, Validator};
use serde::Serialize;
use std::future::Future;

/// Validate any serializable object with [json-schema](https://json-schema.org/)
pub trait Validate: Serialize + Sized {
    /// Validates this object.
    ///
    /// If the object fails validation, this will return an
    /// [Error::Validation](crate::Error::Validation) which contains a vector of
    /// all of the validation errors.
    ///
    /// If you're doing multiple validations, use [Validator::validate], which
    /// will re-use cached schemas.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_validate::Validate;
    /// use stac::Item;
    ///
    /// let mut item = Item::new("an-id");
    /// # tokio_test::block_on(async {
    /// item.validate().await.unwrap();
    /// });
    /// ```
    fn validate(&self) -> impl Future<Output = Result<()>> {
        async {
            let validator = Validator::new().await;
            validator.validate(self).await
        }
    }
}

impl<T: Serialize> Validate for T {}
