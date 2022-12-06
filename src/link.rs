//! Links.

use crate::media_type;
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

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
    /// let item = stac::read("data/simple-item.json").unwrap();
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
    /// let mut item = stac::read("data/simple-item.json").unwrap();
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
    /// let item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.link("root").unwrap();
    /// ```
    fn link(&self, rel: &str) -> Option<&Link> {
        self.links().iter().find(|link| link.rel == rel)
    }

    /// Removes and returns the first link with the given rel type.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let mut item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.remove_link("root").unwrap();
    /// ```
    fn remove_link(&mut self, rel: &str) -> Option<Link> {
        if let Some(i) = self.links().iter().position(|link| link.rel == rel) {
            Some(self.links_mut().remove(i))
        } else {
            None
        }
    }

    /// Returns this object's root link.
    ///
    /// This is the first link with a rel="root".
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.root_link().unwrap();
    /// ```
    fn root_link(&self) -> Option<&Link> {
        self.links().iter().find(|link| link.is_root())
    }

    /// Sets this object's root link.
    ///
    /// This link will have a JSON media type. This removes and returns any
    /// existing root link.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Links, Item};
    /// let mut item = Item::new("an-id");
    /// assert!(item.set_root_link("an/href", "The title of the link".to_string()).is_none());
    /// assert!(item.root_link().is_some());
    /// ```
    fn set_root_link(
        &mut self,
        href: impl ToString,
        title: impl Into<Option<String>>,
    ) -> Option<Link> {
        let previous_root_link = self.remove_link(ROOT_REL);
        self.links_mut().push(Link {
            href: href.to_string(),
            rel: ROOT_REL.to_string(),
            r#type: Some(media_type::JSON.to_string()),
            title: title.into(),
            additional_fields: Default::default(),
        });
        previous_root_link
    }

    /// Returns this object's self link.
    ///
    /// This is the first link with a rel="self".
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.root_link().unwrap();
    /// ```
    fn self_link(&self) -> Option<&Link> {
        self.links().iter().find(|link| link.is_self())
    }

    /// Sets this object's self link.
    ///
    /// This link will have a JSON media type. This removes and returns any
    /// existing self link.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Links, Item};
    /// let mut item = Item::new("an-id");
    /// assert!(item.set_self_link("an/href", "The title of the link".to_string()).is_none());
    /// assert!(item.self_link().is_some());
    /// ```
    fn set_self_link(
        &mut self,
        href: impl ToString,
        title: impl Into<Option<String>>,
    ) -> Option<Link> {
        let previous_self_link = self.remove_link(SELF_REL);
        self.links_mut().push(Link {
            href: href.to_string(),
            rel: SELF_REL.to_string(),
            r#type: Some(media_type::JSON.to_string()),
            title: title.into(),
            additional_fields: Default::default(),
        });
        previous_self_link
    }

    /// Returns this object's parent link.
    ///
    /// This is the first link with a rel="parent".
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let item = stac::read("data/simple-item.json").unwrap();
    /// let link = item.parent_link().unwrap();
    /// ```
    fn parent_link(&self) -> Option<&Link> {
        self.links().iter().find(|link| link.is_parent())
    }

    /// Sets this object's parent link.
    ///
    /// This link will have a JSON media type. This removes and returns any
    /// existing parent link.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Links, Item};
    /// let mut item = Item::new("an-id");
    /// assert!(item.set_parent_link("an/href", "The title of the link".to_string()).is_none());
    /// assert!(item.parent_link().is_some());
    /// ```
    fn set_parent_link(
        &mut self,
        href: impl ToString,
        title: impl Into<Option<String>>,
    ) -> Option<Link> {
        let previous_parent_link = self.remove_link(PARENT_REL);
        self.links_mut().push(Link {
            href: href.to_string(),
            rel: PARENT_REL.to_string(),
            r#type: Some(media_type::JSON.to_string()),
            title: title.into(),
            additional_fields: Default::default(),
        });
        previous_parent_link
    }

    /// Returns an iterator over this object's child links.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Links;
    /// let collection = stac::read("data/collection.json").unwrap();
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
    /// let collection = stac::read("data/collection.json").unwrap();
    /// let links: Vec<_> = collection.iter_item_links().collect();
    /// ```
    fn iter_item_links(&self) -> Box<dyn Iterator<Item = &Link> + '_> {
        Box::new(self.links().iter().filter(|link| link.is_item()))
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

    /// Creates a new root link with JSON media type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Link, media_type};
    /// let root = Link::root("an-href");
    /// assert!(root.is_root());
    /// assert_eq!(root.r#type.as_ref().unwrap(), media_type::JSON);
    /// ```
    pub fn root(href: impl ToString) -> Link {
        Link::new(href, ROOT_REL).json()
    }

    /// Creates a new child link with JSON media type.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Link, media_type};
    /// let root = Link::child("an-href");
    /// assert!(root.is_child());
    /// assert_eq!(root.r#type.as_ref().unwrap(), media_type::JSON);
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
    /// let root = Link::item("an-href");
    /// assert!(root.is_item());
    /// assert_eq!(root.r#type.as_ref().unwrap(), media_type::JSON);
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
    /// let root = Link::parent("an-href");
    /// assert!(root.is_parent());
    /// assert_eq!(root.r#type.as_ref().unwrap(), media_type::JSON);
    /// ```
    pub fn parent(href: impl ToString) -> Link {
        Link::new(href, PARENT_REL).json()
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
        self.is_child() || self.is_item() || self.is_parent() || self.is_root() || self.is_self()
    }
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
        use crate::{Item, Link, Links};

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
    }
}
