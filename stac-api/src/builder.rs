use crate::Error;
use serde::Serialize;
use stac::Link;
use std::str::FromStr;
use url::{ParseError, Url};

/// Builds urls on a root url.
///
/// # Examples
///
/// ```
/// # use stac_api::UrlBuilder;
/// let url_builder = UrlBuilder::new("http://stac-api-rs.test/api/v1").unwrap();
/// assert_eq!(
///     url_builder.items("my-great-collection").unwrap().as_str(),
///     "http://stac-api-rs.test/api/v1/collections/my-great-collection/items"
/// );
/// ```
#[derive(Clone, Debug)]
pub struct UrlBuilder {
    root: Url,
    collections: Url,
    collections_with_slash: Url,
    conformance: Url,
    service_desc: Url,
    search: Url,
}

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
#[derive(Clone, Debug)]
pub struct LinkBuilder(UrlBuilder);

impl UrlBuilder {
    /// Creates a new url builder.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::UrlBuilder;
    /// let url_builder = UrlBuilder::new("http://stac-api-rs.test").unwrap();
    /// ```
    pub fn new(url: &str) -> Result<UrlBuilder, ParseError> {
        let root: Url = if url.ends_with('/') {
            url.parse()?
        } else {
            format!("{}/", url).parse()?
        };
        Ok(UrlBuilder {
            collections: root.join("collections")?,
            collections_with_slash: root.join("collections/")?,
            conformance: root.join("conformance")?,
            service_desc: root.join("api")?,
            search: root.join("search")?,
            root,
        })
    }

    /// Returns the root url.
    ///
    /// The root url always has a trailing slash, even if the builder was
    /// created without one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::UrlBuilder;
    /// let url_builder = UrlBuilder::new("http://stac-api-rs.test").unwrap();
    /// assert_eq!(url_builder.root().as_str(), "http://stac-api-rs.test/");
    /// ```
    pub fn root(&self) -> &Url {
        &self.root
    }

    /// Returns the collections url.
    ///
    /// This url is created when the builder is created, so it can't error.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::UrlBuilder;
    /// let url_builder = UrlBuilder::new("http://stac-api-rs.test").unwrap();
    /// assert_eq!(url_builder.collections().as_str(), "http://stac-api-rs.test/collections");
    /// ```
    pub fn collections(&self) -> &Url {
        &self.collections
    }

    /// Returns a collection url.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::UrlBuilder;
    /// let url_builder = UrlBuilder::new("http://stac-api-rs.test").unwrap();
    /// assert_eq!(
    ///     url_builder.collection("a-collection").unwrap().as_str(),
    ///     "http://stac-api-rs.test/collections/a-collection"
    /// );
    /// ```
    pub fn collection(&self, id: &str) -> Result<Url, ParseError> {
        self.collections_with_slash.join(id)
    }

    /// Returns an items url.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::UrlBuilder;
    /// let url_builder = UrlBuilder::new("http://stac-api-rs.test").unwrap();
    /// assert_eq!(
    ///     url_builder.items("a-collection").unwrap().as_str(),
    ///     "http://stac-api-rs.test/collections/a-collection/items"
    /// );
    /// ```
    pub fn items(&self, id: &str) -> Result<Url, ParseError> {
        self.collections_with_slash.join(&format!("{}/items", id))
    }

    /// Returns the conformance url.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::UrlBuilder;
    /// let url_builder = UrlBuilder::new("http://stac-api-rs.test").unwrap();
    /// assert_eq!(
    ///     url_builder.conformance().as_str(),
    ///     "http://stac-api-rs.test/conformance"
    /// );
    /// ```
    pub fn conformance(&self) -> &Url {
        &self.conformance
    }

    /// Returns the service-desc url.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::UrlBuilder;
    /// let url_builder = UrlBuilder::new("http://stac-api-rs.test").unwrap();
    /// assert_eq!(
    ///     url_builder.service_desc().as_str(),
    ///     "http://stac-api-rs.test/api"
    /// );
    /// ```
    pub fn service_desc(&self) -> &Url {
        &self.service_desc
    }

    /// Returns the search url.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::UrlBuilder;
    /// let url_builder = UrlBuilder::new("http://stac-api-rs.test").unwrap();
    /// assert_eq!(
    ///     url_builder.search().as_str(),
    ///     "http://stac-api-rs.test/search"
    /// );
    /// ```
    pub fn search(&self) -> &Url {
        &self.search
    }
}

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
        Link::root(self.0.root())
    }

    /// Returns a root's self link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.root_to_self();
    /// assert_eq!(link.rel, "self");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/");
    /// ```
    pub fn root_to_self(&self) -> Link {
        Link::self_(self.0.root())
    }

    /// Returns the collections' self link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.collections_to_self();
    /// assert_eq!(link.rel, "self");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections");
    /// ```
    pub fn collections_to_self(&self) -> Link {
        Link::self_(self.0.collections())
    }

    /// Returns a service-desc link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.service_desc();
    /// assert_eq!(link.rel, "service-desc");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/api");
    /// ```
    pub fn service_desc(&self) -> Link {
        Link::new(self.0.service_desc(), "service-desc")
    }

    /// Returns a conformance link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.conformance();
    /// assert_eq!(link.rel, "conformance");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/conformance");
    /// ```
    pub fn conformance(&self) -> Link {
        Link::new(self.0.conformance(), "conformance")
    }

    /// Returns a collections link.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.collections();
    /// assert_eq!(link.rel, "data");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections");
    /// ```
    pub fn collections(&self) -> Link {
        Link::new(self.0.collections(), "data")
    }

    /// Returns an child link for a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.root_to_collection("an-id").unwrap();
    /// assert_eq!(link.rel, "child");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections/an-id");
    /// ```
    pub fn root_to_collection(&self, id: &str) -> Result<Link, ParseError> {
        self.0.collection(id).map(Link::child)
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
    /// let link = link_builder.collection_to_parent();
    /// assert_eq!(link.rel, "parent");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/");
    /// ```
    pub fn collection_to_parent(&self) -> Link {
        Link::parent(self.0.root())
    }

    /// Returns a self link for a collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.collection_to_self("an-id").unwrap();
    /// assert_eq!(link.rel, "self");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections/an-id");
    /// ```
    pub fn collection_to_self(&self, id: &str) -> Result<Link, ParseError> {
        self.0.collection(id).map(Link::self_)
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
    pub fn next_items<S>(&self, id: &str, parameters: S) -> Result<Link, Error>
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
    pub fn prev_items<S>(&self, id: &str, parameters: S) -> Result<Link, Error>
    where
        S: Serialize,
    {
        self.items_with_rel(id, parameters, "prev")
    }

    fn items_with_rel<S>(&self, id: &str, parameters: S, rel: impl ToString) -> Result<Link, Error>
    where
        S: Serialize,
    {
        self.0
            .items(id)
            .map_err(Error::from)
            .and_then(|url| {
                serde_urlencoded::to_string(parameters)
                    .map(|query| (url, query))
                    .map_err(Error::from)
            })
            .map(|(mut url, query)| {
                if !query.is_empty() {
                    url.set_query(Some(&query))
                }
                Link::new(url, rel).geojson()
            })
    }

    /// Returns a link from a collection to its items.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::LinkBuilder;
    /// let link_builder: LinkBuilder = "http://stac-api-rs.test/api/v1".parse().unwrap();
    /// let link = link_builder.collection_to_items("an-id").unwrap();
    /// assert_eq!(link.rel, "items");
    /// assert_eq!(link.href, "http://stac-api-rs.test/api/v1/collections/an-id/items");
    /// ```
    pub fn collection_to_items(&self, id: &str) -> Result<Link, Error> {
        self.items_with_rel(id, (), "items")
    }
}

impl FromStr for UrlBuilder {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        UrlBuilder::new(s)
    }
}

impl FromStr for LinkBuilder {
    type Err = ParseError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        s.parse::<UrlBuilder>()
            .map(|url_builder| LinkBuilder(url_builder))
    }
}
