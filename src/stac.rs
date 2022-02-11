use crate::{Error, Href, Item, Link, Object, PathBufHref, Read, Reader, Render, Write};
use std::collections::{HashMap, HashSet, VecDeque};

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

/// An iterator over a Stac's handles.
#[derive(Debug)]
pub struct Handles<'a, R: Read> {
    handles: VecDeque<Handle>,
    pub(crate) stac: &'a mut Stac<R>,
}

/// An iterator over a Stac's objects.
#[derive(Debug)]
pub struct Objects<'a, R: Read>(pub Handles<'a, R>);

/// An iterator over a Stac's items.
#[derive(Debug)]
pub struct Items<'a, R: Read>(pub Objects<'a, R>);

#[derive(Debug, Default)]
pub(crate) struct Node {
    pub(crate) object: Option<Object>,
    pub(crate) children: HashSet<Handle>,
    pub(crate) parent: Option<Handle>,
    pub(crate) href: Option<Href>,
}

impl Stac<Reader> {
    /// Creates a new Stac `rooted` at the provided object and configured to use
    /// a [Reader].
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let catalog = stac::read("data/catalog.json").unwrap();
    /// let (stac, handle) = Stac::with_root(catalog).unwrap();
    /// ```
    pub fn with_root<O>(object: O) -> Result<(Stac<Reader>, Handle), Error>
    where
        O: Into<Object>,
    {
        Stac::with_root_and_reader(object, Reader::default())
    }

    /// Creates a new `Stac` with the provided object and configured to use a
    /// [Reader].
    ///
    /// If the object has a root link, that link will be used as the root node.
    /// Otherwise, the provided object will be used as a root.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Catalog};
    /// let catalog = Catalog::new("an-id");
    /// let stac = Stac::new(catalog);
    /// ```
    pub fn new<O>(object: O) -> Result<(Stac<Reader>, Handle), Error>
    where
        O: Into<Object>,
    {
        let object = object.into();
        if let Some(link) = object.links().iter().find(|link| link.is_root()) {
            let mut href = Href::new(&link.href);
            if let Some(base) = object.href.as_ref() {
                href = base.join(href)?;
                if &href == base {
                    return Stac::with_root(object);
                }
            }
            let (mut stac, _) = Stac::read(href)?;
            let handle = stac.add_object(object)?;
            Ok((stac, handle))
        } else {
            Stac::with_root(object)
        }
    }

    /// Reads a STAC object and returns a `Stac` and a handle to that object.
    ///
    /// If the object has a `root` link, that link will be used as the root
    /// node. Otherwise, the read object will be used.
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
        let object = crate::read(href)?;
        Stac::new(object)
    }
}

impl<R: Read> Stac<R> {
    /// Creates a new Stac rooted at the provided object and with the provided
    /// reader.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Reader};
    /// let catalog = stac::read("data/catalog.json").unwrap();
    /// let (stac, handle) = Stac::with_root_and_reader(catalog, Reader::default()).unwrap();
    /// ```
    pub fn with_root_and_reader<O>(object: O, reader: R) -> Result<(Stac<R>, Handle), Error>
    where
        O: Into<Object>,
    {
        let object = object.into();
        let handle = ROOT_HANDLE;
        let href = object.href.clone();
        let mut hrefs = HashMap::new();
        if let Some(href) = href.as_ref().cloned() {
            let _ = hrefs.insert(href, handle);
        }
        let node = Node {
            object: None,
            children: HashSet::new(),
            parent: None,
            href,
        };
        let mut stac = Stac {
            reader,
            nodes: vec![node],
            hrefs,
        };
        stac.link_and_add(handle, object)?;
        Ok((stac, handle))
    }

    /// Returns a handle to the root object.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (stac, _) = Stac::read("data/catalog.json").unwrap();
    /// let handle = stac.root();
    /// ```
    pub fn root(&self) -> Handle {
        ROOT_HANDLE
    }

    /// Returns a reference to an object in the stac.
    ///
    /// This method will read an unresolved node if need be.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let child_handle = stac.children(root).unwrap().next().unwrap();
    /// let child = stac.get(child_handle).unwrap();
    /// ```
    pub fn get(&mut self, handle: Handle) -> Result<&Object, Error> {
        self.ensure_resolved(handle)?;
        Ok(self
            .node(handle)
            .object
            .as_ref()
            .expect("node has been resolved"))
    }

    /// Returns a mutable reference to an object in the stac.
    ///
    /// This method will read an unresolved node if need be.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let child_handle = stac.children(root).unwrap().next().unwrap();
    /// let child = stac.get_mut(child_handle).unwrap();
    /// ```
    pub fn get_mut(&mut self, handle: Handle) -> Result<&mut Object, Error> {
        self.ensure_resolved(handle)?;
        Ok(self
            .node_mut(handle)
            .object
            .as_mut()
            .expect("node has been resolved"))
    }

    /// Without modifying the tree structure, removes an object from a node.
    ///
    /// The node remains in the tree, and can will be re-read when needed again.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let catalog = stac.take(root).unwrap();
    /// assert_eq!(catalog.id(), "examples");
    /// let catalog = stac.get(root).unwrap();
    /// assert_eq!(catalog.id(), "examples");
    /// ```
    pub fn take(&mut self, handle: Handle) -> Result<Object, Error> {
        self.ensure_resolved(handle)?;
        Ok(self
            .node_mut(handle)
            .object
            .take()
            .expect("node is resolved"))
    }

    /// Sets the href for the given node.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// stac.set_href(root, "a/new/href").unwrap();
    /// ```
    pub fn set_href<T: Into<Href>>(&mut self, handle: Handle, href: T) -> Result<(), Error> {
        self.nodes
            .get_mut(handle.0)
            .ok_or(Error::InvalidHandle(handle))?
            .href = Some(href.into());
        Ok(())
    }

    /// Gets the href for the given node.
    pub fn href(&self, handle: Handle) -> Result<Option<&Href>, Error> {
        Ok(self
            .nodes
            .get(handle.0)
            .ok_or(Error::InvalidHandle(handle))?
            .href
            .as_ref())
    }

    /// Adds an object to this Stac.
    pub fn add_object(&mut self, object: Object) -> Result<Handle, Error> {
        let handle = if let Some(href) = object.href.as_ref().cloned() {
            self.lookup_or_create(None, href)?
        } else {
            self.add_node(Node::default())
        };
        self.link_and_add(handle, object)?;
        Ok(handle)
    }

    /// Iterates over this stac's objects.
    pub fn objects(&mut self) -> Objects<'_, R> {
        Objects(self.handles())
    }

    /// Iterates over this stac's items.
    pub fn items(&mut self) -> Items<'_, R> {
        Items(self.objects())
    }

    /// Children
    pub fn children(&self, handle: Handle) -> Result<impl Iterator<Item = Handle> + '_, Error> {
        Ok(self
            .nodes
            .get(handle.0)
            .ok_or(Error::InvalidHandle(handle))?
            .children
            .iter()
            .cloned())
    }

    /// Writes this stac.
    pub fn write<T, W>(&mut self, renderer: &T, writer: &W) -> Result<(), Error>
    where
        T: Render,
        W: Write,
    {
        for result in renderer.render(self)? {
            let object = result?;
            writer.write(object)?;
        }
        Ok(())
    }

    /// Iterate over this stac's handles.
    pub fn handles(&mut self) -> Handles<'_, R> {
        let mut handles = VecDeque::new();
        handles.push_front(self.root());
        Handles {
            handles,
            stac: self,
        }
    }

    /// Returns the parent of the provided node.
    pub fn parent(&self, handle: Handle) -> Result<Option<Handle>, Error> {
        Ok(self
            .nodes
            .get(handle.0)
            .ok_or(Error::InvalidHandle(handle))?
            .parent)
    }

    pub(crate) fn ensure_resolved(&mut self, handle: Handle) -> Result<(), Error> {
        if self
            .nodes
            .get(handle.0)
            .ok_or(Error::InvalidHandle(handle))?
            .object
            .is_none()
        {
            self.resolve(handle)?;
        }
        Ok(())
    }

    fn resolve(&mut self, handle: Handle) -> Result<(), Error> {
        let object = if let Some(href) = self.node_mut(handle).href.clone() {
            self.reader.read(href)?
        } else {
            return Err(Error::UnresolvableNode);
        };
        self.link_and_add(handle, object)
    }

    fn link_and_add(&mut self, handle: Handle, object: Object) -> Result<(), Error> {
        self.link(handle, &object)?;
        self.node_mut(handle).object = Some(object);
        Ok(())
    }

    fn link(&mut self, handle: Handle, object: &Object) -> Result<(), Error> {
        for link in object.links() {
            if link.is_child() || link.is_item() {
                self.add_child_link(handle, object.href.as_ref(), link)?;
            } else if link.is_parent() {
                self.add_parent_link(handle, object.href.as_ref(), link)?;
            }
        }
        Ok(())
    }

    fn add_child_link(
        &mut self,
        parent: Handle,
        base: Option<&Href>,
        child: &Link,
    ) -> Result<(), Error> {
        let child = self.lookup_or_create(base, Href::new(&child.href))?;
        self.add_child(parent, child)
    }

    fn add_parent_link(
        &mut self,
        child: Handle,
        base: Option<&Href>,
        parent: &Link,
    ) -> Result<(), Error> {
        let parent = self.lookup_or_create(base, Href::new(&parent.href))?;
        self.add_child(parent, child)
    }

    fn add_child(&mut self, parent: Handle, child: Handle) -> Result<(), Error> {
        let _ = self.node_mut(parent).children.insert(child);
        let child = self.node_mut(child);
        if child
            .parent
            .map(|previous| previous != parent)
            .unwrap_or(false)
        {
            unimplemented!()
        }
        child.parent = Some(parent);
        Ok(())
    }

    fn lookup_or_create(&mut self, base: Option<&Href>, mut href: Href) -> Result<Handle, Error> {
        if let Some(base) = base {
            href = base.join(href)?;
        }
        if let Some(handle) = self.hrefs.get(&href) {
            Ok(*handle)
        } else {
            self.create_node(href)
        }
    }

    fn create_node(&mut self, href: Href) -> Result<Handle, Error> {
        let node = Node {
            object: None,
            children: HashSet::new(),
            parent: None,
            href: Some(href),
        };
        Ok(self.add_node(node))
    }

    fn add_node(&mut self, node: Node) -> Handle {
        let handle = Handle(self.nodes.len());
        if let Some(href) = node.href.as_ref().cloned() {
            // TODO should we error if there's already a node w/ this href?
            let _ = self.hrefs.insert(href, handle);
        }
        self.nodes.push(node);
        handle
    }

    pub(crate) fn node(&self, handle: Handle) -> &Node {
        &self.nodes[handle.0]
    }

    pub(crate) fn node_mut(&mut self, handle: Handle) -> &mut Node {
        &mut self.nodes[handle.0]
    }
}

impl<R: Read> Iterator for Handles<'_, R> {
    type Item = Result<Handle, Error>;

    fn next(&mut self) -> Option<Result<Handle, Error>> {
        if let Some(handle) = self.handles.pop_front() {
            match self.stac.ensure_resolved(handle) {
                Ok(()) => {
                    let node = self.stac.node(handle);
                    self.handles.extend(&node.children);
                    Some(Ok(handle))
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}

impl<R: Read> Iterator for Objects<'_, R> {
    type Item = Result<Object, Error>;

    fn next(&mut self) -> Option<Result<Object, Error>> {
        self.0.next().map(|result| {
            result.map(|handle| {
                self.0
                    .stac
                    .node_mut(handle)
                    .object
                    .take()
                    .expect("should be resolved by the handles iterator")
            })
        })
    }
}

impl<R: Read> Iterator for Items<'_, R> {
    type Item = Result<Item, Error>;

    fn next(&mut self) -> Option<Result<Item, Error>> {
        if let Some(result) = self.0.next() {
            match result {
                Ok(object) => {
                    if let Some(item) = object.into_item() {
                        Some(Ok(item))
                    } else {
                        self.next()
                    }
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Stac;

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
    fn into_objects() {
        let (mut stac, _) = Stac::read("data/catalog.json").unwrap();
        let objects = stac.objects().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(objects.len(), 6);
    }

    #[test]
    fn into_items() {
        let (mut stac, _) = Stac::read("data/catalog.json").unwrap();
        let items = stac.items().collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(items.len(), 2);
    }
}
