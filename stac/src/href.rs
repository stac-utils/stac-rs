use url::Url;

/// Implemented by all three STAC objects, the [Href] trait allows getting and setting an object's href.
///
/// Though the href isn't part of the data structure, it is useful to know where a given STAC object was read from.
/// Objects created from scratch don't have an href.
///
/// # Examples
///
/// ```
/// use stac::{Item, Href};
/// let item = Item::new("an-id");
/// assert!(item.href().is_none());
/// let item: Item = stac::read("data/simple-item.json").unwrap();
/// assert!(item.href().is_some());
/// ```
pub trait Href {
    /// Gets this object's href.
    fn href(&self) -> Option<&str>;

    /// Sets this object's href.
    fn set_href(&mut self, href: impl ToString);
}

/// Parses an href into a [Url] if the scheme is `http` or `https`.
///
/// Otherwise, returns `None`. This is useful for determining whether
/// a given href should be opened with a local filesystem reader or
/// [reqwest](https://docs.rs/reqwest/latest/reqwest/).
///
/// # Examples
///
/// ```
/// assert!(stac::href_to_url("C:\\\\data").is_none());
/// assert!(stac::href_to_url("http://stac-rs.test").is_some());
/// ```
pub fn href_to_url(href: &str) -> Option<Url> {
    if let Ok(url) = Url::parse(href) {
        if url.scheme().starts_with("http") {
            Some(url)
        } else {
            None
        }
    } else {
        None
    }
}
