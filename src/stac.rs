//! An arena-based tree implementation for STAC catalogs.
//!
//! # Reading existing catalogs
//!
//! This example uses a small example catalog adapted from the example Landsat collection in the GeoTrellis repository.
//! It is modeled after [PySTAC's quickstart](https://pystac.readthedocs.io/en/stable/quickstart.html).
//!
//! Use `Stac::read` as a simple way to read a Catalog:
//!
//! ```
//! use stac::Stac;
//! let (mut stac, catalog) = Stac::read("docs/example-catalog/catalog.json").unwrap();
//! let object = stac.get(catalog).unwrap();
//! println!("ID: {}", object.id());
//! println!("Title: {}", object.as_catalog().unwrap().title.as_ref().unwrap());
//! println!("Description: {}", object.as_catalog().unwrap().description);
//! ```
//!
//! ```text
//! ID: landsat-stac-collection-catalog
//! Title: STAC for Landsat data
//! Description: STAC for Landsat data
//! ```
//!
//! # Crawling children
//!
//! A `Stac` handle can be used to fetch the handles of all children of a catalog (or collection):
//!
//! ```
//! # use stac::Stac;
//! # let (mut stac, catalog) = Stac::read("docs/example-catalog/catalog.json").unwrap();
//! println!("Collection ids:");
//! for child in catalog.children(&stac).unwrap() {
//!     println!("- {}", stac.get(child).unwrap().id());
//! }
//! ```
//! ```text
//! Collection ids:
//! - landsat-8-l1
//! ```
//!
//! To fetch a specific child, use the `find_child` method:
//!
//! ```
//! # use stac::Stac;
//! # let (mut stac, catalog) = Stac::read("docs/example-catalog/catalog.json").unwrap();
//! let collection = catalog.find_child(&mut stac, |child| child.id() == "landsat-8-l1").unwrap().unwrap();
//! ```
//!
//! Note that the `Stac` object is a lazily-evaluated cache, so objects are not read into the `Stac` until asked for, and they are persisted inside the `Stac` so they are only read once.
//!
//! # Crawling items
//!
//! To get the handles for all items in a collection or catalog, use the `items()` method:
//!
//! ```
//! # use stac::Stac;
//! # let (mut stac, catalog) = Stac::read("docs/example-catalog/catalog.json").unwrap();
//! let collection = catalog.find_child(&mut stac, |child| child.id() == "landsat-8-l1").unwrap().unwrap();
//! for item in collection.items(&stac).unwrap() {
//!     println!("- {}", stac.get(item).unwrap().id());
//! }
//! ```
//! ```text
//! - LC80140332018166LGN00
//! - LC80150322018141LGN00
//! - LC80150332018189LGN00
//! - LC80300332018166LGN00
//! ```

use crate::{Error, Href, Item, Link, Object, Read, Reader};
use std::collections::{HashMap, VecDeque};

/// An arena-based tree for accessing STAC catalogs.
#[derive(Debug)]
pub struct Stac<R: Read> {
    reader: R,
    nodes: Vec<Node>,
    hrefs: HashMap<Href, Handle>,
}

/// A pointer to a STAC object in a `Stac` tree.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Handle(usize);

/// An iterator over a Stac's objects.
#[derive(Debug)]
pub struct Objects<R: Read> {
    handles: VecDeque<Handle>,
    stac: Stac<R>,
}

/// An iterator over a Stac's items.
#[derive(Debug)]
pub struct Items<R: Read>(Objects<R>);

#[derive(Debug)]
struct Node {
    object: Option<Object>,
    children: Vec<Handle>,
    items: Vec<Handle>,
    parent: Option<Handle>,
    root: Option<Handle>,
    href: Option<Href>,
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
        if !self.get_node(handle)?.is_resolved() {
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
        if !self.get_node_mut(handle)?.is_resolved() {
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
        let href = Href::new(href)?;
        if let Some(handle) = self.hrefs.get(&href) {
            Ok(*handle)
        } else {
            let object = self.reader.read(href)?;
            self.add_object(object)
        }
    }

    /// Converts this Stac into an iterator over all objects, starting at the provided handle.
    ///
    /// Objects above the handle in the tree will not be included.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Stac;
    /// let (stac, catalog) = Stac::read("data/catalog.json").unwrap();
    /// for result in stac.into_objects(catalog) {
    ///     let object = result.unwrap();
    ///     println!("{}", object.id());
    /// }
    /// ```
    pub fn into_objects(self, handle: Handle) -> Objects<R> {
        let mut handles = VecDeque::new();
        handles.push_front(handle);
        Objects {
            stac: self,
            handles,
        }
    }

    /// Converts this Stac into an iterator over all items, starting at the provided handle.
    ///
    /// Items above the handle in the tree will not be included.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Stac;
    /// let (stac, catalog) = Stac::read("data/catalog.json").unwrap();
    /// for result in stac.into_items(catalog) {
    ///     let item = result.unwrap();
    ///     println!("{}", item.id);
    /// }
    /// ```
    pub fn into_items(self, handle: Handle) -> Items<R> {
        Items(self.into_objects(handle))
    }

    fn add_object(&mut self, mut object: Object) -> Result<Handle, Error> {
        let links = self.add_links(&object)?;
        let href = object.href.take();
        if let Some(handle) = href.as_ref().and_then(|href| self.hrefs.get(href).cloned()) {
            self.update_node_unchecked(handle, object, links);
            Ok(handle)
        } else {
            Ok(self.add_node(Node {
                href,
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
                    .push(self.add_link(link, object.href.as_ref())?);
            } else if link.is_item() {
                links.items.push(self.add_link(link, object.href.as_ref())?);
            } else if link.is_parent() {
                // TODO what do do if there are multiple parents?
                links.parent = Some(self.add_link(link, object.href.as_ref())?);
            } else if link.is_root() {
                // TODO what do do if there are multiple roots?
                links.root = Some(self.add_link(link, object.href.as_ref())?);
            }
        }
        Ok(links)
    }

    fn add_link(&mut self, link: &Link, base: Option<&Href>) -> Result<Handle, Error> {
        let href = if let Some(base) = base {
            base.join(&link.href)?
        } else {
            Href::new(&link.href)?
        };
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
                .as_ref()
                .ok_or(Error::UnresolvableNode)?
                .clone(),
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

    fn get_node(&self, handle: Handle) -> Result<&Node, Error> {
        self.nodes.get(handle.0).ok_or(Error::InvalidHandle(handle))
    }

    fn get_node_mut(&mut self, handle: Handle) -> Result<&mut Node, Error> {
        self.nodes
            .get_mut(handle.0)
            .ok_or(Error::InvalidHandle(handle))
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
    /// # use stac::Stac;
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
    /// # use stac::Stac;
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
    /// # use stac::Stac;
    /// let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
    /// for child in catalog.children(&stac).unwrap() {
    ///     println!("{}", stac.get(child).unwrap().id());
    /// }
    /// ```
    pub fn children<R: Read>(&self, stac: &Stac<R>) -> Result<Vec<Handle>, Error> {
        stac.get_node(*self).map(|node| node.children.clone())
    }

    /// Returns an iterator over this object's items (as handles).
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Stac;
    /// let (mut stac, catalog) = Stac::read("data/catalog.json").unwrap();
    /// for item in catalog.items(&stac).unwrap() {
    ///     println!("{}", stac.get(item).unwrap().id());
    /// }
    /// ```
    pub fn items<R: Read>(&self, stac: &Stac<R>) -> Result<Vec<Handle>, Error> {
        stac.get_node(*self).map(|node| node.items.clone())
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
        stac.get_node(*self).map(|node| node.parent)
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
        stac.get_node(*self).map(|node| node.root)
    }
}

impl Node {
    fn is_resolved(&self) -> bool {
        self.object.is_some()
    }
}

impl<R: Read> Iterator for Objects<R> {
    type Item = Result<Object, Error>;

    fn next(&mut self) -> Option<Result<Object, Error>> {
        if let Some(handle) = self.handles.pop_front() {
            if let Err(err) = self.stac.resolve_unchecked(handle) {
                return Some(Err(err));
            }
            match self.stac.get_node_mut(handle) {
                Ok(node) => {
                    self.handles.extend(&node.children);
                    self.handles.extend(&node.items);
                    Some(Ok(node.object.take().expect("the node should be resolved")))
                }
                Err(err) => Some(Err(err)),
            }
        } else {
            None
        }
    }
}

impl<R: Read> Iterator for Items<R> {
    type Item = Result<Item, Error>;

    fn next(&mut self) -> Option<Result<Item, Error>> {
        match self.0.next() {
            Some(result) => match result {
                Ok(object) => {
                    if object.is_item() {
                        Some(Ok(object.into_item().expect("the object is an item")))
                    } else {
                        self.next()
                    }
                }
                Err(err) => Some(Err(err)),
            },
            None => None,
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
        assert_eq!(catalog.items(&stac).unwrap(), vec![item]);
    }

    #[test]
    fn parent() {
        let (mut stac, item) = Stac::read("data/collectionless-item.json").unwrap();
        let parent = item.parent(&stac).unwrap().unwrap();
        assert_eq!(stac.get(parent).unwrap().id(), "examples");
        assert_eq!(parent.items(&stac).unwrap(), vec![item]);
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
        assert_eq!(collection.items(&stac).unwrap(), vec![item]);
    }

    #[test]
    fn into_objects() {
        let (stac, catalog) = Stac::read("data/catalog.json").unwrap();
        let objects = stac
            .into_objects(catalog)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(objects.len(), 6);
    }

    #[test]
    fn into_items() {
        let (stac, catalog) = Stac::read("data/catalog.json").unwrap();
        let items = stac
            .into_items(catalog)
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(items.len(), 2);
    }
}
