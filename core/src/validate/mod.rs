//! Validate STAC objects with [json-schema](https://json-schema.org/).
//!
//! # Examples
//!
//! Validation is provided via the [Validate] trait:
//!
//! ```
//! use stac::{Item, Validate};
//!
//! # tokio_test::block_on(async {
//! Item::new("an-id").validate().await.unwrap();
//! # })
//! ```
//!
//! If you're working in a blocking context (not async), enable the `blocking` feature and use [ValidateBlocking]:
//!
//! ```
//! #[cfg(feature = "blocking")]
//! {
//!     use stac::{ValidateBlocking, Item};
//!     Item::new("an-id").validate_blocking().unwrap();
//! }
//! ```
//!
//! All fetched schemas are cached, so if you're you're doing multiple
//! validations, you should re-use the same [Validator]:
//!
//! ```
//! # use stac::{Item, Validator};
//! let mut items: Vec<_> = (0..10).map(|n| Item::new(format!("item-{}", n))).collect();
//! # tokio_test::block_on(async {
//! let mut validator = Validator::new().await;
//! for item in items {
//!     validator.validate(&item).await.unwrap();
//! }
//! # })
//! ```
//!
//! [Validator] is cheap to clone, so you are encouraged to validate a large
//! number of objects at the same time if that's your use-case.

use crate::Result;
use serde::Serialize;
use std::future::Future;

#[cfg(feature = "validate-blocking")]
mod blocking;
mod validator;

#[cfg(feature = "validate-blocking")]
pub use blocking::ValidateBlocking;
pub use validator::Validator;

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
    /// use stac::{Item, Validate};
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
