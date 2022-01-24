use crate::{Catalog, Collection, Item, Link};
use serde::de::DeserializeOwned;
use std::slice::Iter;

/// A trait implemented by all three STAC objects (`Catalog`, `Container`, and `Item`).
pub trait Object: DeserializeOwned {
    /// Returns this object's href.
    ///
    /// # Examples
    ///
    /// `Catalog` implements `Object`:
    ///
    /// ```
    /// use stac::{Catalog, Object};
    /// let catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.href(), None);
    /// let catalog: Catalog = stac::read("data/catalog.json").unwrap();
    /// assert!(catalog.href().is_some());
    /// ```
    fn href(&self) -> Option<&str>;

    /// Sets this object's href.
    ///
    /// # Examples
    ///
    /// `Catalog` implements `Object`:
    ///
    /// ```
    /// use stac::{Catalog, Object};
    /// let mut catalog = Catalog::new("an-id");
    /// assert_eq!(catalog.href(), None);
    /// catalog.set_href("anything");
    /// assert_eq!(catalog.href().unwrap(), "anything");
    /// ```
    fn set_href<T: ToString>(&mut self, href: T);

    /// Returns an iterator over this object's links.
    ///
    /// # Example
    ///
    /// `Catalog` implements `Object`:
    ///
    /// ```
    /// use stac::{Catalog, Object};
    /// let catalog: Catalog = stac::read("data/catalog.json").unwrap();
    /// let links: Vec<_> = catalog.iter_links().collect();
    /// ```
    fn iter_links(&self) -> Iter<'_, Link>;
}

impl Object for Catalog {
    fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    fn set_href<T: ToString>(&mut self, href: T) {
        self.href = Some(href.to_string())
    }

    fn iter_links(&self) -> Iter<'_, Link> {
        self.links.iter()
    }
}

impl Object for Collection {
    fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    fn set_href<T: ToString>(&mut self, href: T) {
        self.href = Some(href.to_string())
    }

    fn iter_links(&self) -> Iter<'_, Link> {
        self.links.iter()
    }
}

impl Object for Item {
    fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    fn set_href<T: ToString>(&mut self, href: T) {
        self.href = Some(href.to_string())
    }

    fn iter_links(&self) -> Iter<'_, Link> {
        self.links.iter()
    }
}
