use crate::{Error, Href, Object, ObjectHrefTuple, PathBufHref, Read, Reader};
use std::collections::{HashMap, HashSet};

const ROOT_HANDLE: Handle = Handle(0);

/// An arena-based tree for accessing STAC catalogs.
///
/// A `Stac` is generic over its `reader`, which allows `Stac`s to be configured
/// to use custom readers if needed.
#[derive(Debug)]
pub struct Stac<R: Read> {
    reader: R,
    nodes: Vec<Node>,
    hrefs: HashMap<Href, Handle>,
}

/// A pointer to a STAC object in a [Stac] tree.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Handle(pub usize);

#[derive(Debug, Default)]
struct Node {
    object: Option<Object>,
    children: HashSet<Handle>,
    parent: Option<Handle>,
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
                let handle = stac.add_object(object)?;
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
    /// assert_eq!(stac.object(root).unwrap().id(), "examples");
    /// ```
    pub fn object(&mut self, handle: Handle) -> Result<&Object, Error> {
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
    /// let handle = stac.add_object(Catalog::new("unattached-catalog")).unwrap();
    /// ```
    pub fn add_object<O>(&mut self, object: O) -> Result<Handle, Error>
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
        let child_handle = self.add_object(object)?;
        self.add_child_handle(parent, child_handle);
        Ok(child_handle)
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
            let object = self.object(child)?;
            if filter(object) {
                return Ok(Some(child));
            }
        }
        Ok(None)
    }

    fn add_child_handle(&mut self, parent: Handle, child: Handle) {
        self.node_mut(child).parent = Some(parent);
        let _ = self.node_mut(parent).children.insert(child);
    }

    fn add_node(&mut self) -> Handle {
        let handle = Handle(self.nodes.len());
        self.nodes.push(Node::default());
        handle
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
        for link in object
            .links()
            .iter()
            .filter(|link| link.is_child() || link.is_item())
        {
            let child_href = if let Some(href) = href.as_ref() {
                href.join(&link.href)?
            } else {
                link.href.clone().into()
            };
            let child_handle = if let Some(child_handle) = self.hrefs.get(&child_href) {
                *child_handle
            } else {
                let child_handle = self.add_node();
                self.set_href(child_handle, child_href);
                child_handle
            };
            self.add_child_handle(handle, child_handle);
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

#[cfg(test)]
mod tests {
    use super::Stac;
    use crate::{Catalog, HrefObject, Item, Link};

    #[test]
    fn new() {
        let (mut stac, handle) = Stac::new(Catalog::new("an-id")).unwrap();
        assert_eq!(stac.object(handle).unwrap().id(), "an-id");
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
            .add_object(HrefObject::new(
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
        assert_eq!(stac.object(child).unwrap().id(), "extensions-collection");
    }

    #[test]
    fn read() {
        let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
        let catalog = stac.object(handle).unwrap();
        assert_eq!(catalog.id(), "examples");
    }

    #[test]
    fn read_non_root() {
        let (mut stac, handle) = Stac::read("data/extensions-collection/collection.json").unwrap();
        assert_eq!(stac.object(handle).unwrap().id(), "extensions-collection");
        assert_eq!(stac.object(stac.root()).unwrap().id(), "examples");
    }
}
