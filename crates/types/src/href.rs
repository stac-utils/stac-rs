/// Implemented by all three STAC objects, the [Href] trait allows getting and setting an object's href.
///
/// Though the href isn't part of the data structure, it is useful to know where a given STAC object was read from.
/// Objects created from scratch don't have an href.
///
/// # Examples
///
/// ```
/// use stac::{Item, Href};
///
/// let item = Item::new("an-id");
/// assert!(item.href().is_none());
/// let item: Item = stac::read("examples/simple-item.json").unwrap();
/// assert!(item.href().is_some());
/// ```
pub trait Href {
    /// Gets this object's href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Href, Item};
    ///
    /// let item: Item = stac::read("examples/simple-item.json").unwrap();
    /// assert_eq!(item.href(), Some("examples/simple-item.json"));
    /// ```
    fn href(&self) -> Option<&str>;

    /// Sets this object's href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Item, Href};
    ///
    /// let mut item = Item::new("an-id");
    /// item.set_href("http://stac.test/item.json");
    /// ```
    fn set_href(&mut self, href: impl ToString);

    /// Clears this object's href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Href, Item};
    ///
    /// let mut item: Item = stac::read("examples/simple-item.json").unwrap();
    /// item.clear_href();
    /// assert!(item.href().is_none());
    /// ```
    fn clear_href(&mut self);
}
