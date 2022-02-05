//! An arena-based tree implementation for STAC catalogs.

use crate::{utils, Core, Error, Link, Object, Read, Reader};
use std::collections::HashMap;

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
    href: Option<String>,
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
        if !self.nodes[handle.0].is_resolved() {
            self.resolve(handle)?;
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
        if !self.nodes[handle.0].is_resolved() {
            self.resolve(handle)?;
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
        let (children, items) = self.add_links(&object)?;
        Ok(self.add_node(Node {
            href: object.as_ref().href.clone(),
            object: Some(object),
            children,
            items,
        }))
    }

    fn add_links(&mut self, object: &Object) -> Result<(Vec<Handle>, Vec<Handle>), Error> {
        let mut children = Vec::new();
        let mut items = Vec::new();
        for link in object.links() {
            if link.is_child() {
                children.push(self.add_link(link, object.as_ref().href.clone())?);
            } else if link.is_item() {
                items.push(self.add_link(link, object.as_ref().href.clone())?);
            }
        }
        Ok((children, items))
    }

    fn add_link(&mut self, link: &Link, base: Option<String>) -> Result<Handle, Error> {
        Ok(self.add_node(Node {
            object: None,
            children: Vec::new(),
            items: Vec::new(),
            href: Some(utils::absolute_href(&link.href, base.as_deref())?),
        }))
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

    fn resolve(&mut self, handle: Handle) -> Result<(), Error> {
        let object = self.reader.read(
            self.nodes[handle.0]
                .href
                .as_deref()
                .ok_or(Error::UnresolvableNode)?,
            None,
        )?;
        let (children, items) = self.add_links(&object)?;
        let node = &mut self.nodes[handle.0];
        node.object = Some(object);
        node.children = children;
        node.items = items;
        Ok(())
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
        for handle in self.children(stac) {
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
        for handle in self.items(stac) {
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
    /// for child in catalog.children(&stac) {
    ///     println!("{}", stac.get(child).unwrap().id());
    /// }
    /// ```
    pub fn children<R: Read>(&self, stac: &Stac<R>) -> impl Iterator<Item = Handle> {
        stac.nodes[self.0].children.clone().into_iter()
    }

    /// Returns an iterator over this object's items (as handles).
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{Stac, Core};
    /// let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
    /// for item in catalog.items(&stac) {
    ///     println!("{}", stac.get(item).unwrap().id());
    /// }
    /// ```
    pub fn items<R: Read>(&self, stac: &Stac<R>) -> impl Iterator<Item = Handle> {
        stac.nodes[self.0].items.clone().into_iter()
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
        assert_eq!(catalog.items(&stac).collect::<Vec<_>>(), vec![item]);
    }
}
