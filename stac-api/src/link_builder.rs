use crate::{Error, Result};
use serde::Serialize;
use stac::Link;
use std::str::FromStr;
use url::{ParseError, Url};

/// Build links to endpoints in a STAC API.
///
/// # Examples
///
/// [LinkBuilder] can be parsed from a string:
///
/// ```
/// # use stac_api::LinkBuilder;
/// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
/// ```
///
/// Note that the root will always have a trailing slash, even if you didn't provide one:
///
/// ```
/// # use stac_api::LinkBuilder;
/// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
/// assert_eq!(link_builder.root().href, "http://stac-api-rs.test/api/v1/");
/// ```
#[derive(Debug)]
pub struct LinkBuilder(Url);

impl LinkBuilder {
    /// Returns a root link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let root = link_builder.root();
    /// assert_eq!(root.rel, "root");
    /// assert_eq!(root.href, "http://stac-api-rs.test/api/v1/");
    /// ```
    pub fn root(&self) -> Link {
        Link::root(self.0.as_str())
    }

    /// Returns a root's self link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let root = link_builder.root_self();
    /// assert_eq!(root.rel, "self");
    /// assert_eq!(root.href, "http://stac-api-rs.test/api/v1/");
    /// ```
    pub fn root_self(&self) -> Link {
        Link::self_(self.0.as_str())
    }

    /// Returns an child link for a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.child_collection("an-id").unwrap();
    /// assert_eq!(link.rel, "child");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections/an-id");
    /// ```
    pub fn child_collection(&self, id: &str) -> Result<Link> {
        self.0
            .join(&format!("collections/{}", id))
            .map(Link::child)
            .map_err(Error::from)
    }

    /// Returns a parent link for a collection.
    ///
    /// This is just the root url.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.collection_parent();
    /// assert_eq!(link.rel, "parent");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/");
    /// ```
    pub fn collection_parent(&self) -> Link {
        Link::parent(self.0.as_str())
    }

    /// Returns a self link for a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.collection_self("an-id").unwrap();
    /// assert_eq!(link.rel, "self");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections/an-id");
    /// ```
    pub fn collection_self(&self, id: &str) -> Result<Link> {
        self.0
            .join(&format!("collections/{}", id))
            .map(Link::self_)
            .map_err(Error::from)
    }

    /// Returns an items link for a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.items("an-id", ()).unwrap();
    /// assert_eq!(link.rel, "items");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections/an-id/items");
    /// ```
    pub fn items<S>(&self, id: &str, parameters: S) -> Result<Link>
    where
        S: Serialize,
    {
        self.items_with_rel(id, parameters, "items")
    }

    /// Returns a next items link for a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.next_items("an-id", [("foo", "bar")]).unwrap();
    /// assert_eq!(link.rel, "next");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections/an-id/items?foo=bar");
    /// ```
    pub fn next_items<S>(&self, id: &str, parameters: S) -> Result<Link>
    where
        S: Serialize,
    {
        self.items_with_rel(id, parameters, "next")
    }

    /// Returns a prev items link for a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.prev_items("an-id", [("foo", "bar")]).unwrap();
    /// assert_eq!(link.rel, "prev");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections/an-id/items?foo=bar");
    /// ```
    pub fn prev_items<S>(&self, id: &str, parameters: S) -> Result<Link>
    where
        S: Serialize,
    {
        self.items_with_rel(id, parameters, "prev")
    }

    fn items_with_rel<S>(&self, id: &str, parameters: S, rel: impl ToString) -> Result<Link>
    where
        S: Serialize,
    {
        self.0
            .join(&format!("collections/{}/items", id))
            .map_err(Error::from)
            .and_then(|url| {
                serde_urlencoded::to_string(parameters)
                    .map(|query| (url, query))
                    .map_err(Error::from)
            })
            .map(|(mut url, query)| {
                if !query.is_empty() {
                    url.set_query(Some(&query));
                }
                Link::new(url, rel).geojson()
            })
    }
}

impl FromStr for LinkBuilder {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        if s.ends_with('/') {
            s.parse().map(LinkBuilder)
        } else {
            format!("{}/", s).parse().map(LinkBuilder)
        }
    }
}
