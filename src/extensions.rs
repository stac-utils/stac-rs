use crate::{Catalog, Collection, Item};

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

impl Extensions for Item {
    fn extensions(&self) -> Option<&[String]> {
        self.extensions.as_deref()
    }
}
impl Extensions for Catalog {
    fn extensions(&self) -> Option<&[String]> {
        self.extensions.as_deref()
    }
}

impl Extensions for Collection {
    fn extensions(&self) -> Option<&[String]> {
        self.extensions.as_deref()
    }
}

impl Extensions for &Item {
    fn extensions(&self) -> Option<&[String]> {
        self.extensions.as_deref()
    }
}
impl Extensions for &Catalog {
    fn extensions(&self) -> Option<&[String]> {
        self.extensions.as_deref()
    }
}

impl Extensions for &Collection {
    fn extensions(&self) -> Option<&[String]> {
        self.extensions.as_deref()
    }
}
