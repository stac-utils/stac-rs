use crate::{Result, Validate};
use serde::Serialize;
use tokio::runtime::Builder;

/// Validate any serializable object with [json-schema](https://json-schema.org/)
///
/// This is a blocking alternative to [Validate]
pub trait ValidateBlocking: Validate {
    /// Validates this object.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{ValidateBlocking, Item};
    ///
    /// let mut item = Item::new("an-id");
    /// item.validate_blocking().unwrap();
    /// ```
    fn validate_blocking(&self) -> Result<()> {
        Builder::new_current_thread()
            .enable_all()
            .build()?
            .block_on(self.validate())
    }
}

impl<T: Serialize> ValidateBlocking for T {}

#[cfg(test)]
mod tests {
    use super::ValidateBlocking;
    use crate::{Catalog, Collection, Item};
    use rstest as _;

    #[test]
    fn item() {
        let item = Item::new("an-id");
        item.validate_blocking().unwrap();
    }

    #[test]
    #[cfg(feature = "geo")]
    fn item_with_geometry() {
        use geojson::{Geometry, Value};

        let mut item = Item::new("an-id");
        item.set_geometry(Geometry::new(Value::Point(vec![-105.1, 40.1])))
            .unwrap();
        item.validate_blocking().unwrap();
    }

    #[test]
    fn item_with_extensions() {
        let item: Item =
            crate::read("examples/extensions-collection/proj-example/proj-example.json").unwrap();
        item.validate_blocking().unwrap();
    }

    #[test]
    fn catalog() {
        let catalog = Catalog::new("an-id", "a description");
        catalog.validate_blocking().unwrap();
    }

    #[test]
    fn collection() {
        let collection = Collection::new("an-id", "a description");
        collection.validate_blocking().unwrap();
    }

    #[test]
    fn value() {
        let value: crate::Value = crate::read("examples/simple-item.json").unwrap();
        value.validate_blocking().unwrap();
    }

    #[test]
    fn item_collection() {
        let item = crate::read("examples/simple-item.json").unwrap();
        let item_collection = crate::ItemCollection::from(vec![item]);
        item_collection.validate_blocking().unwrap();
    }
}
