use crate::{Error, Href, Object, ObjectHrefTuple, PathBufHref, Read, Reader};
use indexmap::IndexSet;
use std::collections::{HashMap, VecDeque};

const ROOT_HANDLE: Handle = Handle(0);

/// An arena-based tree for accessing STAC catalogs.
///
/// A `Stac` is generic over its `reader`, which allows `Stac`s to be configured
/// to use custom readers if needed.
#[derive(Debug)]
pub struct Stac<R: Read> {
    reader: R,
    nodes: Vec<Node>,
    free_nodes: Vec<Handle>,
    hrefs: HashMap<Href, Handle>,
}

/// A pointer to a STAC object in a [Stac] tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle(usize);

/// An iterator over a [Stac's](Stac) handles.
#[derive(Debug)]
pub struct Walk<'a, R: Read, F, T>
where
    F: Fn(&mut Stac<R>, Handle) -> Result<T, Error>,
{
    handles: VecDeque<Handle>,
    stac: &'a mut Stac<R>,
    f: F,
    depth_first: bool,
}

#[derive(Debug, Default)]
struct Node {
    object: Option<Object>,
    children: IndexSet<Handle>,
    parent: Option<Handle>,
    root: Option<Handle>,
    href: Option<Href>,
}

impl Stac<Reader> {
    /// Creates a new `Stac` with the provided object and configured to use the
    /// default [Reader].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Catalog};
    /// let catalog = Catalog::new("an-id");
    /// let (stac, handle) = Stac::new(catalog).unwrap();
    /// ```
    pub fn new<O>(object: O) -> Result<(Stac<Reader>, Handle), Error>
    where
        O: Into<ObjectHrefTuple>,
    {
        Stac::new_with_reader(object, Reader::default())
    }

    /// Reads a STAC object with the default [Reader] and returns a `Stac` and a
    /// handle to that object.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Stac;
    /// let (stac, handle) = Stac::read("data/catalog.json").unwrap();
    /// ```
    pub fn read<T>(href: T) -> Result<(Stac<Reader>, Handle), Error>
    where
        T: Into<PathBufHref>,
    {
        let reader = Reader::default();
        let href_object = reader.read(href)?;
        Stac::new_with_reader(href_object, reader)
    }
}

impl<R: Read> Stac<R> {
    /// Creates a new Stac with the provided object and with the provided
    /// [Read].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Reader};
    /// let catalog = stac::read("data/catalog.json").unwrap();
    /// let (stac, handle) = Stac::new_with_reader(catalog, Reader::default()).unwrap();
    /// ```
    pub fn new_with_reader<O>(object: O, reader: R) -> Result<(Stac<R>, Handle), Error>
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

    fn rooted<O>(object: O, reader: R) -> Result<(Stac<R>, Handle), Error>
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

    /// Returns the root handle of this [Stac].
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

    /// Returns a reference to an object in this [Stac].
    ///
    /// This method will resolve the object using its [Href], which requires a mutable reference to the `Stac`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// assert_eq!(stac.get(root).unwrap().id(), "examples");
    /// ```
    pub fn get(&mut self, handle: Handle) -> Result<&Object, Error> {
        self.ensure_resolved(handle)?;
        Ok(self
            .node(handle)
            .object
            .as_ref()
            .expect("should be resolved"))
    }

    /// Returns the parent handle of the node, if one is set.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// assert_eq!(stac.parent(root), None);
    /// let child = stac
    ///     .find_child(root, |object| object.id() == "extensions-collection")
    ///     .unwrap()
    ///     .unwrap();
    /// assert_eq!(stac.parent(child).unwrap(), root);
    /// ```
    pub fn parent(&self, handle: Handle) -> Option<Handle> {
        self.node(handle).parent
    }

    /// Adds an object to the [Stac].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Catalog, Stac};
    /// let (mut stac, root) = Stac::new(Catalog::new("a-catalog")).unwrap();
    /// let handle = stac.add(Catalog::new("unattached-catalog")).unwrap();
    /// ```
    pub fn add<O>(&mut self, object: O) -> Result<Handle, Error>
    where
        O: Into<ObjectHrefTuple>,
    {
        let (object, href) = object.into();
        let handle = href
            .and_then(|href| self.hrefs.get(&href).cloned())
            .unwrap_or_else(|| self.add_node());
        self.set_object(handle, object)?;
        Ok(handle)
    }

    /// Adds an object to the [Stac] as a child of the provided handle.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Item, Catalog, Stac};
    /// let (mut stac, root) = Stac::new(Catalog::new("a-catalog")).unwrap();
    /// let handle = stac.add_child(root, Item::new("an-item")).unwrap();
    /// ```
    pub fn add_child<O>(&mut self, parent: Handle, object: O) -> Result<Handle, Error>
    where
        O: Into<ObjectHrefTuple>,
    {
        let child = self.add(object)?;
        self.connect(parent, child);
        Ok(child)
    }

    /// Finds a child object with a filter function.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// assert_eq!(stac.parent(root), None);
    /// let child = stac
    ///     .find_child(root, |object| object.id() == "extensions-collection")
    ///     .unwrap()
    ///     .unwrap();
    /// ```
    pub fn find_child<F>(&mut self, parent: Handle, filter: F) -> Result<Option<Handle>, Error>
    where
        F: Fn(&Object) -> bool,
    {
        for child in self.node(parent).children.clone() {
            let object = self.get(child)?;
            if filter(object) {
                return Ok(Some(child));
            }
        }
        Ok(None)
    }

    /// Walks a [Stac].
    pub fn walk<F, T>(&mut self, handle: Handle, f: F) -> Walk<'_, R, F, T>
    where
        F: Fn(&mut Stac<R>, Handle) -> Result<T, Error>,
    {
        let mut handles = VecDeque::new();
        handles.push_front(handle);
        Walk {
            handles,
            stac: self,
            f,
            depth_first: false,
        }
    }

    /// Returns all child handles as a ref.
    pub fn children(&self, handle: Handle) -> &IndexSet<Handle> {
        &self.node(handle).children
    }

    /// Removes a node.
    pub fn remove(&mut self, handle: Handle) -> (Option<Object>, Option<Href>) {
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
        (object, href)
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

    fn ensure_resolved(&mut self, handle: Handle) -> Result<(), Error> {
        if self.node(handle).object.is_none() {
            self.resolve(handle)?;
        }
        Ok(())
    }

    fn resolve(&mut self, handle: Handle) -> Result<(), Error> {
        if let Some(href) = self.node(handle).href.as_ref() {
            let href_object = self.reader.read(href)?;
            self.set_object(handle, href_object)?;
        }
        Ok(())
    }

    fn set_object<O>(&mut self, handle: Handle, object: O) -> Result<(), Error>
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
                self.connect(handle, other);
            } else if link.is_parent() {
                // TODO what to do if there is already a parent?
                self.connect(other, handle);
            } else if link.is_root() {
                // TODO what to do if the root is different?
                self.node_mut(handle).root = Some(other);
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

    fn set_href(&mut self, handle: Handle, href: Href) {
        let _ = self.hrefs.insert(href.clone(), handle);
        self.node_mut(handle).href = Some(href);
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
    F: Fn(&mut Stac<R>, Handle) -> Result<T, Error>,
{
    /// Walk depth-first instead of breadth first.
    pub fn depth_first(mut self) -> Walk<'a, R, F, T> {
        self.depth_first = true;
        self
    }
}

impl<R: Read, F, T> Iterator for Walk<'_, R, F, T>
where
    F: Fn(&mut Stac<R>, Handle) -> Result<T, Error>,
{
    type Item = Result<T, Error>;

    fn next(&mut self) -> Option<Self::Item> {
        self.handles.pop_front().map(|handle| {
            self.stac
                .ensure_resolved(handle)
                .and_then(|()| (self.f)(self.stac, handle))
                .map(|value| {
                    if self.depth_first {
                        for &child in self.stac.children(handle).iter().rev() {
                            self.handles.push_front(child);
                        }
                    } else {
                        self.handles.extend(self.stac.children(handle));
                    }
                    value
                })
        })
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
            .find_child(root, |object| object.id() == "extensions-collection")
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
        let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
        let count = stac
            .walk(handle, |stac, handle| Ok(stac.remove(handle)))
            .count();
        assert_eq!(count, 1)
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
        assert_eq!(*stac.remove(handle).0.unwrap().as_catalog().unwrap(), child);
    }
}
