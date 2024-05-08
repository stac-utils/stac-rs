//! Links.

use crate::{media_type, Error, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use url::Url;

/// Child links.
pub const CHILD_REL: &str = "child";
/// Item link.
pub const ITEM_REL: &str = "item";
/// Parent link.
pub const PARENT_REL: &str = "parent";
/// Root link.
pub const ROOT_REL: &str = "root";
/// Self link.
pub const SELF_REL: &str = "self";
/// Collection link.
pub const COLLECTION_REL: &str = "collection";

/// This object describes a relationship with another entity.
///
/// Data providers are advised to be liberal with the links section, to describe
/// things like the [Catalog](crate::Catalog) an [Item](crate::Item) is in,
/// related `Item`s, parent or child `Item`s (modeled in different ways, like an
/// 'acquisition' or derived data). It is allowed to add additional fields such
/// as a title and type.
///
/// This link structure includes a few fields from the [STAC API
/// specification](https://github.com/radiantearth/stac-api-spec/tree/main/item-search#pagination).
/// Generally we keep STAC API structures in the [stac-api
/// crate](https://github.com/stac-utils/stac-rs/stac-api), but in this case it
/// was simpler to include these attributes in the base [Link] rather to create a new one.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Link {
    /// The actual link in the format of an URL.
    ///
    /// Relative and absolute links are both allowed.
    pub href: String,

    /// Relationship between the current document and the linked document.
    ///
    /// See the chapter on ["Relation
    /// types"](https://github.com/radiantearth/stac-spec/blob/master/item-spec/item-spec.md#relation-types)
    /// in the STAC spec for more information.
    pub rel: String,

    /// [Media type](crate::media_type) of the referenced entity.
    #[serde(rename = "type")]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub r#type: Option<String>,

    /// A human readable title to be used in rendered displays of the link.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,

    /// The HTTP method of the request, usually GET or POST. Defaults to GET.
    ///
    /// From the STAC API spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,

    /// A dictionary of header values that must be included in the next request
    ///
    /// From the STAC API spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub headers: Option<Map<String, Value>>,

    /// A JSON object containing fields/values that must be included in the body
    /// of the next request.
    ///
    /// From the STAC API spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<Map<String, Value>>,

    /// If true, the headers/body fields in the next link must be merged into
    /// the original request and be sent combined in the next request. Defaults
    /// to false
    ///
    /// From the STAC API spec.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge: Option<bool>,

    /// Additional fields on the link.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

/// Implemented by any object that has links.
pub trait Links {
    /// Returns a reference to this object's links.
    ///
    /// # Examples
    ///
    /// [Value](crate::Value) implements Links:
    ///
    /// ```
    /// use stac::Links;
    /// let item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// let links = item.links();
    /// ```
    fn links(&self) -> &[Link];

    /// Returns a mutable reference to this object's links.
    ///
    /// # Examples
    ///
    /// [Value](crate::Value) implements Links:
    ///
    /// ```
    /// use stac::Links;
    /// let mut item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// let links = item.links_mut();
    /// links.clear();
    /// ```
    fn links_mut(&mut self) -> &mut Vec<Link>;

    /// Returns the first link with the given rel type.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.link("root").unwrap();
    /// ```
    fn link(&self, rel: &str) -> Option<&Link> {
        self.links().iter().find(|link| link.rel == rel)
    }

    /// Sets a link of the given rel type.
    ///
    /// This will remove all other links of that rel type, so should only be
    /// used for e.g. "root", not e.g. "child".
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Links, Link};
    /// let mut item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// item.set_link(Link::root("a/href"));
    /// ```
    fn set_link(&mut self, link: Link) {
        self.links_mut().retain(|l| l.rel != link.rel);
        self.links_mut().push(link)
    }

    /// Returns this object's root link.
    ///
    /// This is the first link with a rel="root".
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.root_link().unwrap();
    /// ```
    fn root_link(&self) -> Option<&Link> {
        self.links().iter().find(|link| link.is_root())
    }

    /// Returns this object's self link.
    ///
    /// This is the first link with a rel="self".
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.root_link().unwrap();
    /// ```
    fn self_link(&self) -> Option<&Link> {
        self.links().iter().find(|link| link.is_self())
    }

    /// Returns this object's parent link.
    ///
    /// This is the first link with a rel="parent".
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let item: stac::Item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.parent_link().unwrap();
    /// ```
    fn parent_link(&self) -> Option<&Link> {
        self.links().iter().find(|link| link.is_parent())
    }

    /// Returns an iterator over this object's child links.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let collection: stac::Collection = stac::read("data/collection.json").unwrap();
    /// let links: Vec<_> = collection.iter_child_links().collect();
    /// ```
    fn iter_child_links(&self) -> Box<dyn Iterator<Item = &Link> + '_> {
        Box::new(self.links().iter().filter(|link| link.is_child()))
    }

    /// Returns an iterator over this object's item links.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let collection: stac::Collection = stac::read("data/collection.json").unwrap();
    /// let links: Vec<_> = collection.iter_item_links().collect();
    /// ```
    fn iter_item_links(&self) -> Box<dyn Iterator<Item = &Link> + '_> {
        Box::new(self.links().iter().filter(|link| link.is_item()))
    }

    /// Makes all relative links absolute with respect to an href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Links, Catalog, Error, Href};
    ///
    /// let mut catalog: stac::Catalog = stac::read("data/catalog.json").unwrap();
    /// assert!(!catalog.root_link().unwrap().is_absolute());
    /// catalog.make_relative_links_absolute("data/catalog.json").unwrap();
    /// assert!(catalog.root_link().unwrap().is_absolute());
    /// ```
    fn make_relative_links_absolute(&mut self, href: impl ToString) -> Result<()> {
        let href = make_absolute(href.to_string(), None)?;
        for link in self.links_mut() {
            link.href = make_absolute(std::mem::take(&mut link.href), Some(&href))?;
        }
        Ok(())
    }

    /// Makes all absolute links relative with respect to an href.
    ///
    /// If they do not share a root, the link will be made absolute.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Links, Catalog, Error, Href};
    ///
    /// let mut catalog: stac::Catalog = stac::read("data/catalog.json").unwrap();
    /// assert!(!catalog.root_link().unwrap().is_absolute());
    /// catalog.make_relative_links_absolute("data/catalog.json").unwrap();
    /// assert!(catalog.root_link().unwrap().is_absolute());
    /// catalog.make_absolute_links_relative("data/catalog.json").unwrap();
    /// assert!(catalog.root_link().unwrap().is_relative());
    /// ```
    fn make_absolute_links_relative(&mut self, href: impl ToString) -> Result<()> {
        let href = make_absolute(href.to_string(), None)?;
        for link in self.links_mut() {
            let absolute_link_href = make_absolute(std::mem::take(&mut link.href), Some(&href))?;
            link.href = make_relative(&absolute_link_href, &href);
        }
        Ok(())
    }

    /// Removes all relative links.
    ///
    /// This can be useful e.g. if you're relocating a STAC object, but it
    /// doesn't have a href, so the relative links wouldn't make any sense.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Catalog, Links, Link};
    /// let mut catalog = Catalog::new("an-id", "a description");
    /// catalog.links.push(Link::new("./child.json", "child"));
    /// catalog.remove_relative_links();
    /// assert!(catalog.links.is_empty());
    /// ```
    fn remove_relative_links(&mut self) {
        self.links_mut().retain(|link| link.is_absolute())
    }

    /// Removes all structural links.
    ///
    /// Useful if you're, e.g., going to re-populate the structural links as a
    /// part of serving items with a STAC API.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Catalog, Links, Link};
    /// let mut catalog = Catalog::new("an-id", "a description");
    /// catalog.links.push(Link::self_("http://stac.test/catalog.json"));
    /// catalog.remove_structural_links();
    /// assert!(catalog.links.is_empty());
    /// ```
    fn remove_structural_links(&mut self) {
        self.links_mut().retain(|link| !link.is_structural())
    }
}

impl Link {
    /// Creates a new link with the provided href and rel type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "a-rel");
    /// assert_eq!(link.href, "an-href");
    /// assert_eq!(link.rel, "a-rel");
    /// ```
    pub fn new(href: impl ToString, rel: impl ToString) -> Link {
        Link {
            href: href.to_string(),
            rel: rel.to_string(),
            r#type: None,
            title: None,
            method: None,
            headers: None,
            body: None,
            merge: None,
            additional_fields: Map::new(),
        }
    }

    /// Sets this link's media type to JSON.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Link, media_type};
    /// let link = Link::new("a/href", "rel-type").json();
    /// assert_eq!(link.r#type.unwrap(), media_type::JSON);
    /// ```
    pub fn json(mut self) -> Link {
        self.r#type = Some(media_type::JSON.to_string());
        self
    }

    /// Returns true if this link's media type is JSON.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Link, media_type};
    /// let link = Link::new("a/href", "rel-type").json();
    /// assert!(link.is_json());
    /// ```
    pub fn is_json(&self) -> bool {
        self.r#type
            .as_ref()
            .map(|t| t == media_type::JSON)
            .unwrap_or(false)
    }

    /// Sets this link's media type to GeoJSON.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Link, media_type};
    /// let link = Link::new("a/href", "rel-type").geojson();
    /// assert_eq!(link.r#type.unwrap(), media_type::GEOJSON);
    /// ```
    pub fn geojson(mut self) -> Link {
        self.r#type = Some(media_type::GEOJSON.to_string());
        self
    }

    /// Returns true if this link's media type is GeoJSON.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Link, media_type};
    /// let link = Link::new("a/href", "rel-type").geojson();
    /// assert!(link.is_geojson());
    /// ```
    pub fn is_geojson(&self) -> bool {
        self.r#type
            .as_ref()
            .map(|t| t == media_type::GEOJSON)
            .unwrap_or(false)
    }

    /// Sets this link's media type.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Link, media_type};
    /// let link = Link::new("a/href", "rel-type").r#type(media_type::GEOJSON.to_string());
    /// assert_eq!(link.r#type.unwrap(), media_type::GEOJSON);
    /// ```
    pub fn r#type(mut self, r#type: impl Into<Option<String>>) -> Link {
        self.r#type = r#type.into();
        self
    }

    /// Sets this link's title.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Link;
    /// let link = Link::new("a/href", "rel-type").title("a title".to_string());
    /// assert_eq!(link.title.unwrap(), "a title");
    /// ```
    pub fn title(mut self, title: impl Into<Option<String>>) -> Link {
        self.title = title.into();
        self
    }

    /// Creates a new root link with JSON media type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Link, media_type};
    /// let link = Link::root("an-href");
    /// assert!(link.is_root());
    /// assert_eq!(link.r#type.as_ref().unwrap(), media_type::JSON);
    /// ```
    pub fn root(href: impl ToString) -> Link {
        Link::new(href, ROOT_REL).json()
    }

    /// Creates a new self link with JSON media type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Link, media_type};
    /// let link = Link::self_("an-href");
    /// assert!(link.is_self());
    /// assert_eq!(link.r#type.as_ref().unwrap(), media_type::JSON);
    /// ```
    pub fn self_(href: impl ToString) -> Link {
        Link::new(href, SELF_REL).json()
    }

    /// Creates a new child link with JSON media type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Link, media_type};
    /// let link = Link::child("an-href");
    /// assert!(link.is_child());
    /// assert_eq!(link.r#type.as_ref().unwrap(), media_type::JSON);
    /// ```
    pub fn child(href: impl ToString) -> Link {
        Link::new(href, CHILD_REL).json()
    }

    /// Creates a new item link with JSON media type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Link, media_type};
    /// let link = Link::item("an-href");
    /// assert!(link.is_item());
    /// assert_eq!(link.r#type.as_ref().unwrap(), media_type::JSON);
    /// ```
    pub fn item(href: impl ToString) -> Link {
        Link::new(href, ITEM_REL).json()
    }

    /// Creates a new parent link with JSON media type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Link, media_type};
    /// let link = Link::parent("an-href");
    /// assert!(link.is_parent());
    /// assert_eq!(link.r#type.as_ref().unwrap(), media_type::JSON);
    /// ```
    pub fn parent(href: impl ToString) -> Link {
        Link::new(href, PARENT_REL).json()
    }

    /// Creates a new collection link with JSON media type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Link, media_type};
    /// let link = Link::collection("an-href");
    /// assert!(link.is_collection());
    /// assert_eq!(link.r#type.as_ref().unwrap(), media_type::JSON);
    /// ```
    pub fn collection(href: impl ToString) -> Link {
        Link::new(href, COLLECTION_REL).json()
    }

    /// Returns true if this link's rel is `"item"`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "item");
    /// assert!(link.is_item());
    /// let link = Link::new("an-href", "not-an-item");
    /// assert!(!link.is_item());
    /// ```
    pub fn is_item(&self) -> bool {
        self.rel == ITEM_REL
    }

    /// Returns true if this link's rel is `"child"`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "child");
    /// assert!(link.is_child());
    /// let link = Link::new("an-href", "not-a-child");
    /// assert!(!link.is_child());
    /// ```
    pub fn is_child(&self) -> bool {
        self.rel == CHILD_REL
    }

    /// Returns true if this link's rel is `"parent"`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "parent");
    /// assert!(link.is_parent());
    /// let link = Link::new("an-href", "not-a-parent");
    /// assert!(!link.is_parent());
    /// ```
    pub fn is_parent(&self) -> bool {
        self.rel == PARENT_REL
    }

    /// Returns true if this link's rel is `"root"`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "root");
    /// assert!(link.is_root());
    /// let link = Link::new("an-href", "not-a-root");
    /// assert!(!link.is_root());
    /// ```
    pub fn is_root(&self) -> bool {
        self.rel == ROOT_REL
    }

    /// Returns true if this link's rel is `"self"`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "self");
    /// assert!(link.is_self());
    /// let link = Link::new("an-href", "not-a-self");
    /// assert!(!link.is_self());
    /// ```
    pub fn is_self(&self) -> bool {
        self.rel == SELF_REL
    }

    /// Returns true if this link's rel is `"collection"`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "collection");
    /// assert!(link.is_collection());
    /// let link = Link::new("an-href", "not-a-collection");
    /// assert!(!link.is_collection());
    /// ```
    pub fn is_collection(&self) -> bool {
        self.rel == COLLECTION_REL
    }

    /// Returns true if this link is structural (i.e. not child, parent, item,
    /// root, or self).
    ///
    /// Also includes some API structural link types such as "data",
    /// "conformance", "items", and "search".
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Link;
    /// let link = Link::new("an-href", "self");
    /// assert!(link.is_structural());
    /// let link = Link::new("an-href", "child");
    /// assert!(link.is_structural());
    /// let link = Link::new("an-href", "not-a-root");
    /// assert!(!link.is_structural());
    pub fn is_structural(&self) -> bool {
        self.is_child()
            || self.is_item()
            || self.is_parent()
            || self.is_root()
            || self.is_self()
            || self.is_collection()
            || self.rel == "data"
            || self.rel == "conformance"
            || self.rel == "items"
            || self.rel == "search"
            || self.rel == "service-desc"
            || self.rel == "service-doc"
            || self.rel == "next"
            || self.rel == "prev"
    }

    /// Returns true if this link's href is an absolute path or url.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Link;
    ///
    /// assert!(Link::new("/a/local/path/item.json", "rel").is_absolute());
    /// assert!(Link::new("http://stac-rs.test/item.json", "rel").is_absolute());
    /// assert!(!Link::new("./not/an/absolute/path", "rel").is_absolute());
    /// ```
    pub fn is_absolute(&self) -> bool {
        is_absolute(&self.href)
    }

    /// Returns true if this link's href is a relative path.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Link;
    ///
    /// assert!(!Link::new("/a/local/path/item.json", "rel").is_relative());
    /// assert!(!Link::new("http://stac-rs.test/item.json", "rel").is_relative());
    /// assert!(Link::new("./not/an/absolute/path", "rel").is_relative());
    /// ```
    pub fn is_relative(&self) -> bool {
        !is_absolute(&self.href)
    }

    /// Sets the method attribute on this link.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Link;
    /// let link = Link::new("href", "rel").method("GET");
    /// ```
    pub fn method(mut self, method: impl ToString) -> Link {
        self.method = Some(method.to_string());
        self
    }

    /// Sets the body attribute on this link.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Link;
    /// use serde_json::json;
    ///
    /// let link = Link::new("href", "rel").body(json!({"foo": "bar"})).unwrap();
    /// ```
    pub fn body<T: Serialize>(mut self, body: T) -> Result<Link> {
        match serde_json::to_value(body)? {
            Value::Object(body) => {
                self.body = Some(body);
                Ok(self)
            }
            value => Err(Error::IncorrectType {
                actual: value.to_string(),
                expected: "object".to_string(),
            }),
        }
    }
}

fn is_absolute(href: &str) -> bool {
    Url::parse(href).is_ok() || href.starts_with('/')
}

fn make_absolute(href: String, base: Option<&str>) -> Result<String> {
    // TODO if we make this interface public, make this an impl Option
    if is_absolute(&href) {
        Ok(href)
    } else if let Some(base) = base {
        if let Ok(base) = Url::parse(base) {
            base.join(&href)
                .map(|url| url.to_string())
                .map_err(Error::from)
        } else {
            let (base, _) = base.split_at(base.rfind('/').unwrap_or(0));
            if base.is_empty() {
                Ok(normalize_path(&href))
            } else {
                Ok(normalize_path(&format!("{}/{}", base, href)))
            }
        }
    } else {
        std::fs::canonicalize(href)
            .map(|p| p.to_string_lossy().into_owned())
            .map_err(Error::from)
    }
}

fn make_relative(href: &str, base: &str) -> String {
    // Cribbed from `Url::make_relative`
    let mut relative = String::new();

    fn extract_path_filename(s: &str) -> (&str, &str) {
        let last_slash_idx = s.rfind('/').unwrap_or(0);
        let (path, filename) = s.split_at(last_slash_idx);
        if filename.is_empty() {
            (path, "")
        } else {
            (path, &filename[1..])
        }
    }

    let (base_path, base_filename) = extract_path_filename(base);
    let (href_path, href_filename) = extract_path_filename(href);

    let mut base_path = base_path.split('/').peekable();
    let mut href_path = href_path.split('/').peekable();

    while base_path.peek().is_some() && base_path.peek() == href_path.peek() {
        let _ = base_path.next();
        let _ = href_path.next();
    }

    for base_path_segment in base_path {
        if base_path_segment.is_empty() {
            break;
        }

        if !relative.is_empty() {
            relative.push('/');
        }

        relative.push_str("..");
    }

    for href_path_segment in href_path {
        if relative.is_empty() {
            relative.push_str("./");
        } else {
            relative.push('/');
        }

        relative.push_str(href_path_segment);
    }

    if !relative.is_empty() || base_filename != href_filename {
        if href_filename.is_empty() {
            relative.push('/');
        } else {
            if relative.is_empty() {
                relative.push_str("./");
            } else {
                relative.push('/');
            }
            relative.push_str(href_filename);
        }
    }

    relative
}

fn normalize_path(path: &str) -> String {
    let mut parts = if path.starts_with('/') {
        Vec::new()
    } else {
        vec![""]
    };
    for part in path.split('/') {
        match part {
            "." => {}
            ".." => {
                let _ = parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

#[cfg(test)]
mod tests {
    use super::Link;

    #[test]
    fn new() {
        let link = Link::new("an-href", "a-rel");
        assert_eq!(link.href, "an-href");
        assert_eq!(link.rel, "a-rel");
        assert!(link.r#type.is_none());
        assert!(link.title.is_none());
    }

    #[test]
    fn skip_serializing() {
        let link = Link::new("an-href", "a-rel");
        let value = serde_json::to_value(link).unwrap();
        assert!(value.get("type").is_none());
        assert!(value.get("title").is_none());
    }

    mod links {
        use crate::{Catalog, Item, Link, Links};

        #[test]
        fn link() {
            let mut item = Item::new("an-item");
            assert!(item.link("root").is_none());
            item.links.push(Link::new("an-href", "root"));
            assert!(item.link("root").is_some());
        }

        #[test]
        fn root() {
            let mut item = Item::new("an-item");
            assert!(item.root_link().is_none());
            item.links.push(Link::new("an-href", "root"));
            assert!(item.root_link().is_some());
        }

        #[test]
        fn self_() {
            let mut item = Item::new("an-item");
            assert!(item.self_link().is_none());
            item.links.push(Link::new("an-href", "self"));
            assert!(item.self_link().is_some());
        }

        #[test]
        fn make_relative_links_absolute_path() {
            let mut catalog: Catalog = crate::read("data/catalog.json").unwrap();
            catalog
                .make_relative_links_absolute("data/catalog.json")
                .unwrap();
            for link in catalog.links() {
                assert!(link.is_absolute());
            }
        }

        #[test]
        fn make_relative_links_absolute_url() {
            let mut catalog: Catalog = crate::read("data/catalog.json").unwrap();
            catalog
                .make_relative_links_absolute("http://stac-rs.test/catalog.json")
                .unwrap();
            for link in catalog.links() {
                assert!(link.is_absolute());
            }
            assert_eq!(
                catalog.root_link().unwrap().href,
                "http://stac-rs.test/catalog.json"
            );
        }

        #[test]
        fn make_absolute_links_relative_path() {
            let mut catalog: Catalog = crate::read("data/catalog.json").unwrap();
            catalog
                .make_relative_links_absolute("data/catalog.json")
                .unwrap();
            catalog.make_absolute_links_relative("data/").unwrap();
            for link in catalog.links() {
                if !link.is_self() {
                    assert!(link.is_relative(), "{}", link.href);
                }
            }
        }

        #[test]
        fn make_absolute_links_relative_url() {
            let mut catalog: Catalog = crate::read("data/catalog.json").unwrap();
            catalog
                .make_relative_links_absolute("http://stac-rs.test/catalog.json")
                .unwrap();
            catalog
                .make_absolute_links_relative("http://stac-rs.test/")
                .unwrap();
            assert_eq!(catalog.root_link().unwrap().href, "./catalog.json");
        }

        #[test]
        fn remove_relative_links() {
            let mut catalog = Catalog::new("an-id", "a description");
            catalog.links.push(Link::new("./child.json", "child"));
            catalog.links.push(Link::new("/child.json", "child"));
            catalog
                .links
                .push(Link::new("http://stac-rs.test/child.json", "child"));
            catalog.remove_relative_links();
            assert_eq!(catalog.links.len(), 2);
        }
    }
}
