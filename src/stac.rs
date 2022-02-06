//! An arena-based tree implementation for STAC catalogs.

use crate::{utils, Core, Error, Link, Object, Read, Reader};
use std::{collections::HashMap, vec::IntoIter};

/// An arena-based tree for accessing STAC catalogs.
#[derive(Debug)]
pub struct Stac<R: Read> {
    reader: R,
    nodes: Vec<Node>,
    hrefs: HashMap<String, Handle>,
}

/// A pointer to a STAC object in a `Stac` tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Handle(usize);

#[derive(Debug)]
struct Node {
    object: Option<Object>,
    children: Vec<Handle>,
    items: Vec<Handle>,
    parent: Option<Handle>,
    root: Option<Handle>,
    href: Option<String>,
}

#[derive(Debug, Default)]
struct Links {
    children: Vec<Handle>,
    items: Vec<Handle>,
    parent: Option<Handle>,
    root: Option<Handle>,
}

impl Stac<Reader> {
    /// Reads a STAC object and returns a `Stac` and a handle to that object.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Stac;
    /// let (stac, handle) = Stac::read("data/catalog.json").unwrap();
    /// ```
    pub fn read(href: &str) -> Result<(Stac<Reader>, Handle), Error> {
        let mut stac = Stac::default();
        let handle = stac.add_via_href(href)?;
        Ok((stac, handle))
    }
}

impl<R: Read> Stac<R> {
    /// Gets a reference to an object in a `Stac`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Stac;
    /// let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
    /// let catalog = stac.get(handle).unwrap();
    /// ```
    pub fn get(&mut self, handle: Handle) -> Result<&Object, Error> {
        if !self
            .nodes
            .get(handle.0)
            .ok_or(Error::InvalidHandle(handle))?
            .is_resolved()
        {
            self.resolve_unchecked(handle)?;
        }
        Ok(self.nodes[handle.0]
            .object
            .as_ref()
            .expect("node should be resolved"))
    }

    /// Gets a mutable reference to an object in a `Stac`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Stac;
    /// let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
    /// let catalog = stac.get_mut(handle).unwrap();
    /// ```
    pub fn get_mut(&mut self, handle: Handle) -> Result<&mut Object, Error> {
        if !self
            .nodes
            .get(handle.0)
            .ok_or(Error::InvalidHandle(handle))?
            .is_resolved()
        {
            self.resolve_unchecked(handle)?;
        }
        Ok(self.nodes[handle.0]
            .object
            .as_mut()
            .expect("node should be resolved"))
    }

    /// Moves this `Stac` into a new one with a the provided reader.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Stac, Reader};
    /// let stac = Stac::default();
    /// let stac = stac.with_reader(Reader::default());
    /// ```
    pub fn with_reader<T: Read>(self, reader: T) -> Stac<T> {
        Stac {
            reader,
            nodes: self.nodes,
            hrefs: self.hrefs,
        }
    }

    /// Add an object to the `Stac` via an href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Stac;
    /// let mut stac = Stac::default();
    /// let catalog = stac.add_via_href("data/catalog.json").unwrap();
    /// ```
    pub fn add_via_href(&mut self, href: &str) -> Result<Handle, Error> {
        let href = utils::absolute_href(href, None)?;
        if let Some(handle) = self.hrefs.get(&href) {
            Ok(*handle)
        } else {
            let object = self.reader.read(&href, None)?;
            self.add_object(object)
        }
    }

    fn add_object(&mut self, object: Object) -> Result<Handle, Error> {
        let links = self.add_links(&object)?;
        if let Some(handle) = object
            .as_ref()
            .href
            .as_deref()
            .and_then(|href| self.hrefs.get(href).cloned())
        {
            self.update_node_unchecked(handle, object, links);
            Ok(handle)
        } else {
            Ok(self.add_node(Node {
                href: object.as_ref().href.clone(),
                object: Some(object),
                children: links.children,
                items: links.items,
                parent: links.parent,
                root: links.root,
            }))
        }
    }

    fn add_links(&mut self, object: &Object) -> Result<Links, Error> {
        let mut links = Links::default();
        for link in object.links() {
            if link.is_child() {
                links
                    .children
                    .push(self.add_link(link, object.as_ref().href.as_deref())?);
            } else if link.is_item() {
                links
                    .items
                    .push(self.add_link(link, object.as_ref().href.as_deref())?);
            } else if link.is_parent() {
                // TODO what do do if there are multiple parents?
                links.parent = Some(self.add_link(link, object.as_ref().href.as_deref())?);
            } else if link.is_root() {
                // TODO what do do if there are multiple roots?
                links.root = Some(self.add_link(link, object.as_ref().href.as_deref())?);
            }
        }
        Ok(links)
    }

    fn add_link(&mut self, link: &Link, base: Option<&str>) -> Result<Handle, Error> {
        let href = utils::absolute_href(&link.href, base)?;
        if let Some(handle) = self.hrefs.get(&href) {
            Ok(*handle)
        } else {
            Ok(self.add_node(Node {
                object: None,
                children: Vec::new(),
                items: Vec::new(),
                // TODO should we set the parent here?
                parent: None,
                root: None,
                href: Some(href),
            }))
        }
    }

    fn add_node(&mut self, node: Node) -> Handle {
        let handle = Handle(self.nodes.len());
        if let Some(href) = node.href.as_ref() {
            if self.hrefs.insert(href.clone(), handle).is_some() {
                // TODO implement
                unimplemented!()
            }
        }
        self.nodes.push(node);
        handle
    }

    fn resolve_unchecked(&mut self, handle: Handle) -> Result<(), Error> {
        let object = self.reader.read(
            self.nodes[handle.0]
                .href
                .as_deref()
                .ok_or(Error::UnresolvableNode)?,
            None,
        )?;
        let links = self.add_links(&object)?;
        self.update_node_unchecked(handle, object, links);
        Ok(())
    }

    fn update_node_unchecked(&mut self, handle: Handle, object: Object, links: Links) {
        let node = &mut self.nodes[handle.0];
        node.object = Some(object);
        node.children = links.children;
        node.items = links.items;
        node.parent = links.parent;
        node.root = links.root;
    }
}

impl Default for Stac<Reader> {
    fn default() -> Stac<Reader> {
        Stac {
            reader: Reader::default(),
            nodes: Vec::new(),
            hrefs: HashMap::new(),
        }
    }
}

impl Handle {
    /// Finds a child and returns its handle.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Core};
    /// let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
    /// let collection = catalog
    ///     .find_child(&mut stac, |child| child.id() == "extensions-collection")
    ///     .unwrap()
    ///     .unwrap();
    /// let collection = stac.get(collection).unwrap();
    /// assert_eq!(collection.id(), "extensions-collection");
    /// ```
    pub fn find_child<F, R>(&self, stac: &mut Stac<R>, f: F) -> Result<Option<Handle>, Error>
    where
        F: Fn(&Object) -> bool,
        R: Read,
    {
        for handle in self.children(stac)? {
            let child = stac.get(handle)?;
            if f(child) {
                return Ok(Some(handle));
            }
        }
        Ok(None)
    }

    /// Finds an item and returns its handle.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Core};
    /// let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
    /// let item = catalog
    ///     .find_item(&mut stac, |item| item.id() == "CS3-20160503_132131_08")
    ///     .unwrap()
    ///     .unwrap();
    /// let item = stac.get(item).unwrap();
    /// assert_eq!(item.id(), "CS3-20160503_132131_08");
    /// ```
    pub fn find_item<F, R>(&self, stac: &mut Stac<R>, f: F) -> Result<Option<Handle>, Error>
    where
        F: Fn(&Object) -> bool,
        R: Read,
    {
        for handle in self.items(stac)? {
            let item = stac.get(handle)?;
            if f(item) {
                return Ok(Some(handle));
            }
        }
        Ok(None)
    }

    /// Returns an iterator over this object's children (as handles).
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Core};
    /// let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
    /// for child in catalog.children(&stac).unwrap() {
    ///     println!("{}", stac.get(child).unwrap().id());
    /// }
    /// ```
    pub fn children<R: Read>(&self, stac: &Stac<R>) -> Result<IntoIter<Handle>, Error> {
        stac.nodes
            .get(self.0)
            .ok_or(Error::InvalidHandle(*self))
            .map(|node| node.children.clone().into_iter())
    }

    /// Returns an iterator over this object's items (as handles).
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Core};
    /// let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
    /// for item in catalog.items(&stac).unwrap() {
    ///     println!("{}", stac.get(item).unwrap().id());
    /// }
    /// ```
    pub fn items<R: Read>(&self, stac: &Stac<R>) -> Result<IntoIter<Handle>, Error> {
        stac.nodes
            .get(self.0)
            .ok_or(Error::InvalidHandle(*self))
            .map(|node| node.items.clone().into_iter())
    }

    /// Returns the parent of this object, if there is one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac};
    /// let (mut stac, item) = Stac::read("data/collectionless-item.json").unwrap();
    /// let catalog = item.parent(&mut stac).unwrap().unwrap();
    /// ```
    pub fn parent<R: Read>(&self, stac: &Stac<R>) -> Result<Option<Handle>, Error> {
        stac.nodes
            .get(self.0)
            .ok_or(Error::InvalidHandle(*self))
            .map(|node| node.parent)
    }

    /// Returns the root of this object, if there is one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac};
    /// let (mut stac, item) = Stac::read("data/collectionless-item.json").unwrap();
    /// let catalog = item.root(&mut stac).unwrap().unwrap();
    /// ```
    pub fn root<R: Read>(&self, stac: &Stac<R>) -> Result<Option<Handle>, Error> {
        stac.nodes
            .get(self.0)
            .ok_or(Error::InvalidHandle(*self))
            .map(|node| node.root)
    }
}

impl Node {
    fn is_resolved(&self) -> bool {
        self.object.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::Stac;
    use crate::Core;

    #[test]
    fn read() {
        let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
        let catalog = stac.get(handle).unwrap();
        assert_eq!(catalog.id(), "examples");
    }

    #[test]
    fn find_child() {
        let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
        let handle = catalog
            .find_child(&mut stac, |child| child.id() == "extensions-collection")
            .unwrap()
            .unwrap();
        let child = stac.get(handle).unwrap();
        assert_eq!(child.id(), "extensions-collection");
    }

    #[test]
    fn find_item() {
        let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
        let handle = catalog
            .find_item(&mut stac, |item| item.id() == "CS3-20160503_132131_08")
            .unwrap()
            .unwrap();
        let item = stac.get(handle).unwrap();
        assert_eq!(item.id(), "CS3-20160503_132131_08");
    }

    #[test]
    fn resolve_children() {
        let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
        let collection = catalog
            .find_child(&mut stac, |child| child.id() == "extensions-collection")
            .unwrap()
            .unwrap();
        let item = collection
            .find_item(&mut stac, |item| item.id() == "proj-example")
            .unwrap()
            .unwrap();
        assert_eq!(stac.get(item).unwrap().id(), "proj-example");
    }

    #[test]
    fn prevent_duplicates() {
        let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
        let item = stac.add_via_href("data/collectionless-item.json").unwrap();
        assert_eq!(
            catalog.items(&stac).unwrap().collect::<Vec<_>>(),
            vec![item]
        );
    }

    #[test]
    fn parent() {
        let (mut stac, item) = Stac::read("data/collectionless-item.json").unwrap();
        let parent = item.parent(&stac).unwrap().unwrap();
        assert_eq!(stac.get(parent).unwrap().id(), "examples");
        assert_eq!(parent.items(&stac).unwrap().collect::<Vec<_>>(), vec![item]);
    }

    #[test]
    fn root() {
        let (mut stac, item) =
            Stac::read("data/extensions-collection/proj-example/proj-example.json").unwrap();
        let root = item.root(&stac).unwrap().unwrap();
        assert_eq!(stac.get(root).unwrap().id(), "examples");
        let collection = root
            .find_child(&mut stac, |child| child.id() == "extensions-collection")
            .unwrap()
            .unwrap();
        assert_eq!(
            collection.items(&stac).unwrap().collect::<Vec<_>>(),
            vec![item]
        );
    }
}
