use crate::{Error, Href, Object, ObjectHrefTuple, PathBufHref, Read, Reader, Result};
use indexmap::IndexSet;
use std::collections::{HashMap, VecDeque};

const ROOT_HANDLE: Handle = Handle(0);

/// An arena-based tree for working with STAC catalogs.
///
/// A `Stac` is generic over [Read], which allows `Stac`s to be configured to
/// use custom readers if needed. Many methods of `Stac` work with an
/// [ObjectHrefTuple], which is a tuple an [Object] and an optional [Href].
/// Since [Object] and [HrefObject](crate::HrefObject) both implement [Into] for
/// [ObjectHrefTuple], this enables `Stac` methods to take objects both with and
/// without hrefs.
///
/// A [Stac] uses [Handles](Handle) to reference objects in the tree. A `Handle`
/// is tied to its `Stac`; using a `Handle` on a `Stac` other than the one that
/// produced it is undefined behavior.
///
/// A `root` link is only used when creating a new `Stac`: if the initial object
/// has a `root` link, it is used to set the root of the `Stac`. After that, all
/// `root` links are ignored.
///
/// # Examples
///
/// ```
/// use stac::{Stac, Catalog};
/// let catalog = Catalog::new("root");
/// let item = stac::read_item("data/simple-item.json").unwrap();
/// let (mut stac, root) = Stac::new(catalog).unwrap();
/// let child = stac.add_child(root, item).unwrap();
/// ```
#[derive(Debug)]
pub struct Stac<R: Read> {
    reader: R,
    nodes: Vec<Node>,
    free_nodes: Vec<Handle>,
    hrefs: HashMap<Href, Handle>,
}

/// A pointer to an [Object] in a [Stac] tree.
///
/// Handles can only be used on the `Stac` that produced them. Using a `Handle`
/// on a different `Stac` is undefined behavior.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle(usize);

/// An iterator over a [Stac's](Stac) [Handles](Handle).
#[derive(Debug)]
pub struct Walk<'a, R: Read, F, T>
where
    F: Fn(&mut Stac<R>, Handle) -> Result<T>,
{
    handles: VecDeque<Handle>,
    stac: &'a mut Stac<R>,
    f: F,
    depth_first: bool,
    strategy: WalkStrategy,
}

#[derive(Debug)]
enum WalkStrategy {
    SkipItems,
    ItemsOnly,
    All,
}

#[derive(Debug, Default)]
struct Node {
    object: Option<Object>,
    children: IndexSet<Handle>,
    parent: Option<Handle>,
    href: Option<Href>,
    is_from_item_link: bool,
}

impl Stac<Reader> {
    /// Creates a new `Stac` with the provided object and configured to use
    /// [Reader].
    ///
    /// Returns a tuple of the `Stac` and the [Handle] to the object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Catalog};
    /// let catalog = Catalog::new("an-id");
    /// let (stac, handle) = Stac::new(catalog).unwrap();
    /// ```
    pub fn new<O>(object: O) -> Result<(Stac<Reader>, Handle)>
    where
        O: Into<ObjectHrefTuple>,
    {
        Stac::new_with_reader(object, Reader::default())
    }

    /// Reads an [HrefObject](crate::HrefObject) with [Reader]
    ///
    /// Returns a tuple of the `Stac` and the [Handle] to that object.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Stac;
    /// let (stac, handle) = Stac::read("data/catalog.json").unwrap();
    /// ```
    pub fn read<T>(href: T) -> Result<(Stac<Reader>, Handle)>
    where
        T: Into<PathBufHref>,
    {
        let reader = Reader::default();
        let href_object = reader.read(href)?;
        Stac::new_with_reader(href_object, reader)
    }
}

impl<R: Read> Stac<R> {
    /// Creates a new `Stac` from the [Object] and [Read].
    ///
    /// Returns a tuple of the `Stac` and the [Handle] to that object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Reader};
    /// let catalog = stac::read("data/catalog.json").unwrap();
    /// let (stac, handle) = Stac::new_with_reader(catalog, Reader::default()).unwrap();
    /// ```
    pub fn new_with_reader<O>(object: O, reader: R) -> Result<(Stac<R>, Handle)>
    where
        O: Into<ObjectHrefTuple>,
    {
        let (object, href) = object.into();
        if let Some(link) = object.root_link() {
            let root_href = if let Some(href) = href.as_ref() {
                href.join(&link.href)?
            } else {
                link.href.clone().into()
            };
            if !href
                .as_ref()
                .map(|href| *href == root_href)
                .unwrap_or(false)
            {
                let root = reader.read(root_href)?;
                let (mut stac, _) = Stac::rooted(root, reader)?;
                let handle = stac.add(object)?;
                return Ok((stac, handle));
            }
        }
        Stac::rooted((object, href), reader)
    }

    fn rooted<O>(object: O, reader: R) -> Result<(Stac<R>, Handle)>
    where
        O: Into<ObjectHrefTuple>,
    {
        let handle = ROOT_HANDLE;
        let node = Node::default();
        let mut stac = Stac {
            reader,
            nodes: vec![node],
            free_nodes: Vec::new(),
            hrefs: HashMap::new(),
        };
        stac.set_object(handle, object)?;
        Ok((stac, handle))
    }

    /// Returns the root [Handle] of this `Stac`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (stac, root) = Stac::read("data/catalog.json").unwrap();
    /// assert_eq!(stac.root(), root);
    /// ```
    pub fn root(&self) -> Handle {
        ROOT_HANDLE
    }

    /// Returns a reference to an [Object] in this `Stac`.
    ///
    /// This method will resolve the object using its [Href], which requires a
    /// mutable reference to the `Stac`. This will return an [Err] if there is
    /// an error while reading the object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// assert_eq!(stac.get(root).unwrap().id(), "examples");
    /// ```
    pub fn get(&mut self, handle: Handle) -> Result<&Object> {
        self.ensure_resolved(handle)?;
        Ok(self
            .node(handle)
            .object
            .as_ref()
            .expect("should be resolved"))
    }

    /// Returns the parent [Handle] of this object, if one is set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// assert_eq!(stac.parent(root), None);
    /// let child = stac
    ///     .find(root, |object| object.id() == "extensions-collection")
    ///     .unwrap()
    ///     .unwrap();
    /// assert_eq!(stac.parent(child).unwrap(), root);
    /// ```
    pub fn parent(&self, handle: Handle) -> Option<Handle> {
        self.node(handle).parent
    }

    /// Adds an [Object] to the [Stac].
    ///
    /// If this object has links, the links will be resolved and the object will
    /// be linked into the tree.
    ///
    /// # Examples
    ///
    /// Adding an unattached object:
    ///
    /// ```
    /// # use stac::{Catalog, Stac};
    /// let (mut stac, root) = Stac::new(Catalog::new("a-catalog")).unwrap();
    /// let handle = stac.add(Catalog::new("unattached-catalog")).unwrap();
    /// ```
    ///
    /// Adding an object that will be linked into the tree:
    ///
    /// ```
    /// # use stac::{Catalog, HrefObject, Stac, Link};
    /// # let (mut stac, root) = Stac::new(Catalog::new("a-catalog")).unwrap();
    /// stac.set_href(root, "rootdir/catalog.json");
    /// let mut catalog = Catalog::new("attached-catalog");
    /// catalog.links.push(Link::parent("../catalog.json"));
    /// let href_object = HrefObject::new(catalog, "rootdir/attached-catalog/catalog.json");
    /// let child = stac.add(href_object).unwrap();
    /// assert_eq!(stac.parent(child).unwrap(), root);
    /// ```
    pub fn add<O>(&mut self, object: O) -> Result<Handle>
    where
        O: Into<ObjectHrefTuple>,
    {
        let (object, href) = object.into();
        let handle = href
            .as_ref()
            .and_then(|href| self.hrefs.get(&href).cloned())
            .unwrap_or_else(|| self.add_node());
        self.set_object(handle, (object, href))?;
        Ok(handle)
    }

    /// Adds an [Object] to the [Stac] as a child of the provided handle.
    ///
    /// If there is a `parent` link on the `Object`, it will be ignored.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Item, Catalog, Link, Stac};
    /// let (mut stac, root) = Stac::new(Catalog::new("a-catalog")).unwrap();
    /// let child = stac.add_child(root, Item::new("an-item")).unwrap();
    /// assert_eq!(stac.parent(child).unwrap(), root);
    ///
    /// let mut second_item = Item::new("second-item");
    /// second_item.links.push(Link::parent("some/other/parent.json"));
    /// let child = stac.add_child(root, second_item).unwrap();
    /// assert_eq!(stac.parent(child).unwrap(), root);
    /// ```
    pub fn add_child<O>(&mut self, parent: Handle, object: O) -> Result<Handle>
    where
        O: Into<ObjectHrefTuple>,
    {
        let child = self.add(object)?;
        self.connect(parent, child);
        Ok(child)
    }

    /// Removes an [Object] from the [Stac].
    ///
    /// Unlinks all parents and children. Note that this will leave the children
    /// unattached.  Returns the [Object] and its [Href], if they exist (one of
    /// them will). Returns an error if you try to remove the root object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Error};
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let child = stac.find(root, |o| o.id() == "extensions-collection").unwrap().unwrap();
    /// let (child, href) = stac.remove(child).unwrap();
    /// assert_eq!(child.unwrap().id(), "extensions-collection");
    /// assert_eq!(href.unwrap().as_str(), "data/extensions-collection/collection.json");
    /// assert!(matches!(stac.remove(root).unwrap_err(), Error::CannotRemoveRoot));
    /// ```
    pub fn remove(&mut self, handle: Handle) -> Result<(Option<Object>, Option<Href>)> {
        if handle == self.root() {
            return Err(Error::CannotRemoveRoot);
        }
        let children = std::mem::take(&mut self.node_mut(handle).children);
        for child in children {
            self.disconnect(handle, child);
        }
        if let Some(parent) = self.node_mut(handle).parent.take() {
            self.disconnect(parent, handle);
        }
        let href = if let Some(href) = self.node_mut(handle).href.take() {
            let _ = self.hrefs.remove(&href);
            Some(href)
        } else {
            None
        };
        self.free_nodes.push(handle);
        let object = self.node_mut(handle).object.take();
        Ok((object, href))
    }

    /// Returns the [Href] of an [Object].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Catalog};
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// assert_eq!(stac.href(root).unwrap().as_str(), "data/catalog.json");
    /// let catalog = stac.add(Catalog::new("unattached")).unwrap();
    /// assert!(stac.href(catalog).is_none());
    /// ```
    pub fn href(&self, handle: Handle) -> Option<&Href> {
        self.node(handle).href.as_ref()
    }

    /// Sets the [Href] of an [Object].
    ///
    /// If the `href` was already assigned to another object in the `Stac`, that
    /// object's href will be cleared.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Catalog, Stac};
    /// let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
    /// assert!(stac.href(root).is_none());
    /// stac.set_href(root, "path/to/the/root.catalog");
    /// assert_eq!(stac.href(root).unwrap().as_str(), "path/to/the/root.catalog");
    /// ```
    ///
    /// Clearing another object's href:
    ///
    /// ```
    /// # use stac::{Catalog, Stac};
    /// let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
    /// let child1 = stac.add_child(root, Catalog::new("child1")).unwrap();
    /// stac.set_href(child1, "a/catalog.json");
    /// assert_eq!(stac.href(child1).unwrap().as_str(), "a/catalog.json");
    /// let child2 = stac.add_child(root, Catalog::new("child2")).unwrap();
    /// stac.set_href(child2, "a/catalog.json");
    /// assert_eq!(stac.href(child2).unwrap().as_str(), "a/catalog.json");
    /// assert!(stac.href(child1).is_none());
    /// ```
    pub fn set_href<H>(&mut self, handle: Handle, href: H)
    where
        H: Into<Href>,
    {
        let href = href.into();
        if let Some(other) = self.hrefs.insert(href.clone(), handle) {
            let _ = self.node_mut(other).href.take();
        }
        if let Some(href) = self.node_mut(handle).href.replace(href) {
            assert_eq!(
                self.hrefs
                    .remove(&href)
                    .expect("there should be an entry in hrefs"),
                handle
            );
        }
    }

    /// Returns a [Walk] iterator, which visits all objects in a [Stac] (by default).
    ///
    /// The `Walk` iterator holds a closure, which can be used to extract values
    /// from the `Stac` or even modify it while walking.
    ///
    /// # Examples
    ///
    /// Collect all object ids:
    ///
    /// ```
    /// # use stac::{Stac, Handle};
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let ids = stac
    ///     .walk(root, |stac, handle| {
    ///         stac.get(handle).map(|object| String::from(object.id()))
    ///     })
    ///     .collect::<Result<Vec<_>, _>>()
    ///     .unwrap();
    /// assert_eq!(ids.len(), 6);
    /// ```
    pub fn walk<F, T>(&mut self, handle: Handle, f: F) -> Walk<'_, R, F, T>
    where
        F: Fn(&mut Stac<R>, Handle) -> Result<T>,
    {
        let mut handles = VecDeque::new();
        handles.push_front(handle);
        Walk {
            handles,
            stac: self,
            f,
            depth_first: false,
            strategy: WalkStrategy::All,
        }
    }

    /// Finds an [Object] in the tree using a filter function.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// assert_eq!(stac.parent(root), None);
    /// let child = stac
    ///     .find(root, |object| object.id() == "extensions-collection")
    ///     .unwrap()
    ///     .unwrap();
    /// assert_eq!(stac.get(child).unwrap().id(), "extensions-collection");
    /// ```
    pub fn find<F>(&mut self, handle: Handle, filter: F) -> Result<Option<Handle>>
    where
        F: Fn(&Object) -> bool,
    {
        self.walk(handle, |stac, handle| {
            let object = stac.get(handle)?;
            Ok((filter(object), handle))
        })
        .filter_map(|result| match result {
            Ok((keep, handle)) => {
                if keep {
                    Some(Ok(handle))
                } else {
                    None
                }
            }
            Err(err) => Some(Err(err)),
        })
        .next()
        .transpose()
    }

    fn connect(&mut self, parent: Handle, child: Handle) {
        self.node_mut(child).parent = Some(parent);
        let _ = self.node_mut(parent).children.insert(child);
    }

    fn disconnect(&mut self, parent: Handle, child: Handle) {
        self.node_mut(child).parent = None;
        let _ = self.node_mut(parent).children.shift_remove(&child);
    }

    fn add_node(&mut self) -> Handle {
        if let Some(handle) = self.free_nodes.pop() {
            handle
        } else {
            let handle = Handle(self.nodes.len());
            self.nodes.push(Node::default());
            handle
        }
    }

    fn ensure_resolved(&mut self, handle: Handle) -> Result<()> {
        if self.node(handle).object.is_none() {
            if let Some(href) = self.node(handle).href.as_ref() {
                let href_object = self.reader.read(href)?;
                self.set_object(handle, href_object)?;
            } else {
                panic!("should not be able to get a node w/o an object or an href")
            }
        }
        Ok(())
    }

    fn set_object<O>(&mut self, handle: Handle, object: O) -> Result<()>
    where
        O: Into<ObjectHrefTuple>,
    {
        let (object, href) = object.into();
        for link in object.links() {
            if !link.is_structural() {
                continue;
            }
            let other_href = if let Some(href) = href.as_ref() {
                href.join(&link.href)?
            } else {
                link.href.clone().into()
            };
            let other = if let Some(other) = self.hrefs.get(&other_href) {
                *other
            } else {
                let other = self.add_node();
                self.set_href(other, other_href);
                other
            };
            if link.is_child() || link.is_item() {
                if link.is_item() {
                    self.node_mut(other).is_from_item_link = true;
                }
                self.connect(handle, other);
            } else if link.is_parent() {
                // TODO what to do if there is already a parent?
                self.connect(other, handle);
            }
        }
        if let Some(href) = href {
            self.set_href(handle, href);
        } else {
            self.node_mut(handle).href = None;
        }
        let node = self.node_mut(handle);
        node.object = Some(object);
        Ok(())
    }

    fn is_item(&self, handle: Handle) -> bool {
        if let Some(object) = self.node(handle).object.as_ref() {
            object.is_item()
        } else {
            self.node(handle).is_from_item_link
        }
    }

    fn node(&self, handle: Handle) -> &Node {
        &self.nodes[handle.0]
    }

    fn node_mut(&mut self, handle: Handle) -> &mut Node {
        &mut self.nodes[handle.0]
    }
}

impl<'a, R: Read, F, T> Walk<'a, R, F, T>
where
    F: Fn(&mut Stac<R>, Handle) -> Result<T>,
{
    /// Walk depth-first instead of breadth first.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let ids = stac
    ///     .walk(root, |stac, handle| {
    ///         stac.get(handle).map(|object| String::from(object.id()))
    ///     });
    /// for result in ids.depth_first() {
    ///     let id = result.unwrap();
    ///     println!("{}", id);
    /// }
    /// ```
    pub fn depth_first(mut self) -> Walk<'a, R, F, T> {
        self.depth_first = true;
        self
    }

    /// Skip items while walking the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let ids = stac
    ///     .walk(root, |stac, handle| {
    ///         stac.get(handle).map(|object| String::from(object.id()))
    ///     })
    ///     .skip_items()
    ///     .collect::<Result<Vec<_>, _>>()
    ///     .unwrap();
    /// assert_eq!(ids.len(), 4);
    /// ```
    pub fn skip_items(mut self) -> Walk<'a, R, F, T> {
        self.strategy = WalkStrategy::SkipItems;
        self
    }

    /// Only stop at items when walking the tree.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let ids = stac
    ///     .walk(root, |stac, handle| {
    ///         stac.get(handle).map(|object| String::from(object.id()))
    ///     })
    ///     .items_only()
    ///     .collect::<Result<Vec<_>, _>>()
    ///     .unwrap();
    /// assert_eq!(ids.len(), 2);
    /// ```
    pub fn items_only(mut self) -> Walk<'a, R, F, T> {
        self.strategy = WalkStrategy::ItemsOnly;
        self
    }
}

impl<R: Read, F, T> Iterator for Walk<'_, R, F, T>
where
    F: Fn(&mut Stac<R>, Handle) -> Result<T>,
{
    type Item = Result<T>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(handle) = self.handles.pop_front() {
            if let Err(err) = self.stac.ensure_resolved(handle) {
                self.handles.clear();
                Some(Err(err))
            } else {
                match (self.f)(self.stac, handle) {
                    Ok(value) => {
                        let mut children = VecDeque::new();
                        for &child in &self.stac.node(handle).children {
                            if !(matches!(self.strategy, WalkStrategy::SkipItems)
                                && self.stac.is_item(child))
                            {
                                if self.depth_first {
                                    children.push_front(child);
                                } else {
                                    children.push_back(child);
                                }
                            }
                        }
                        if self.depth_first {
                            for child in children {
                                self.handles.push_front(child);
                            }
                        } else {
                            self.handles.extend(children)
                        }
                        if !(matches!(self.strategy, WalkStrategy::ItemsOnly)
                            && !self.stac.is_item(handle))
                        {
                            Some(Ok(value))
                        } else {
                            self.next()
                        }
                    }
                    Err(err) => {
                        self.handles.clear();
                        Some(Err(err))
                    }
                }
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Stac;
    use crate::{Catalog, HrefObject, Item, Link};

    #[test]
    fn new() {
        let (mut stac, handle) = Stac::new(Catalog::new("an-id")).unwrap();
        assert_eq!(stac.get(handle).unwrap().id(), "an-id");
    }

    #[test]
    fn link() {
        let mut catalog = Catalog::new("an-id");
        catalog
            .links
            .push(Link::new("./subcatalog/catalog.json", "child"));
        let (mut stac, root_handle) =
            Stac::new(HrefObject::new(catalog, "a/path/catalog.json")).unwrap();
        let handle = stac
            .add(HrefObject::new(
                Catalog::new("child-catalog"),
                "a/path/subcatalog/catalog.json",
            ))
            .unwrap();
        assert_eq!(stac.parent(handle).unwrap(), root_handle);
    }

    #[test]
    fn add_child() {
        let (mut stac, root) = Stac::new(Catalog::new("an-id")).unwrap();
        let item = Item::new("an-id");
        let handle = stac.add_child(root, item).unwrap();
        assert_eq!(stac.parent(handle).unwrap(), root);
    }

    #[test]
    fn find_child() {
        let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
        let child = stac
            .find(root, |object| object.id() == "extensions-collection")
            .unwrap()
            .unwrap();
        assert_eq!(stac.get(child).unwrap().id(), "extensions-collection");
    }

    #[test]
    fn read() {
        let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
        let catalog = stac.get(handle).unwrap();
        assert_eq!(catalog.id(), "examples");
    }

    #[test]
    fn read_non_root() {
        let (mut stac, handle) = Stac::read("data/extensions-collection/collection.json").unwrap();
        assert_eq!(stac.get(handle).unwrap().id(), "extensions-collection");
        assert_eq!(stac.get(stac.root()).unwrap().id(), "examples");
    }

    #[test]
    fn walk() {
        let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
        let ids = stac
            .walk(handle, |stac, handle| {
                stac.get(handle).map(|object| object.id().to_string())
            })
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            ids,
            vec![
                "examples",
                "extensions-collection",
                "sentinel-2",
                "sentinel-2",
                "CS3-20160503_132131_08",
                "proj-example",
            ]
        )
    }

    #[test]
    fn walk_depth_first() {
        let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
        let ids = stac
            .walk(handle, |stac, handle| {
                stac.get(handle).map(|object| object.id().to_string())
            })
            .depth_first()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(
            ids,
            vec![
                "examples",
                "extensions-collection",
                "proj-example",
                "sentinel-2",
                "sentinel-2",
                "CS3-20160503_132131_08",
            ]
        )
    }

    #[test]
    fn walk_remove() {
        let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
        let count = stac
            .walk(root, |stac, handle| {
                if handle != root {
                    let _ = stac.remove(handle)?;
                    Ok(())
                } else {
                    Ok(())
                }
            })
            .count();
        assert_eq!(count, 5);
        assert!(stac.find(root, |o| o.is_collection()).unwrap().is_none());
        assert!(stac.find(root, |o| o.is_item()).unwrap().is_none());
    }

    #[test]
    fn remove_returns_same_object() {
        let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
        let mut child = Catalog::new("child");
        child.links.push(Link::root("../catalog.json"));
        child.links.push(Link::parent("../catalog.json"));
        child.links.push(Link::child("./subcatalog/catlog.json"));
        child.links.push(Link::item("./42/42.json"));
        let handle = stac.add_child(root, child.clone()).unwrap();
        assert_eq!(
            *stac
                .remove(handle)
                .unwrap()
                .0
                .unwrap()
                .as_catalog()
                .unwrap(),
            child
        );
    }
}
