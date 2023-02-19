/// A trait for objects that may have STAC extensions.
pub trait Extensions {
    /// Returns a reference to this object's extensions.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Extensions, Item};
    /// let item = Item::new("an-id");
    /// assert!(item.extensions().is_none());
    /// ```
    fn extensions(&self) -> Option<&[String]>;
}
