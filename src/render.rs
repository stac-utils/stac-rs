use crate::{Error, Handle, Handles, Href, Link, Object, Read, Stac};

/// A trait for anything that can render [Stacs](Stac) to an iterable of objects.
///
/// # Examples
///
/// ```
/// use stac::{Stac, Render, BestPracticesRenderer, Error};
/// let (mut stac, _) = Stac::read("data/catalog.json").unwrap();
/// let renderer = BestPracticesRenderer::new("a/root/directory");
/// let objects = renderer.render(&mut stac).unwrap().collect::<Result<Vec<_>, Error>>().unwrap();
/// assert_eq!(objects.len(), 6);
/// let root = &objects[0];
/// assert_eq!(root.href.as_ref().unwrap().as_str(), "a/root/directory/catalog.json");
/// ```
pub trait Render {
    /// Creates an iterator over rendered [Objects](Object).
    ///
    /// The source [Stac] will be modified in the following ways:
    /// - Each object will be removed from the `Stac`. The tree structure will
    /// be preserved, but each node will lose its resolved [Object]. This is to
    /// prevent unnecessary clones.
    /// - Each node's href will be set to the rendered href.
    ///
    /// # Examples
    ///
    /// [BestPracticesRenderer] implements `Render`:
    ///
    /// ```
    /// # use stac::{Render, BestPracticesRenderer, Stac};
    /// let renderer = BestPracticesRenderer::new("data");
    /// let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
    /// let objects = renderer
    ///     .render(&mut stac)
    ///     .unwrap()
    ///     .collect::<Result<Vec<_>, _>>()
    ///     .unwrap();
    /// ```
    fn render<'stac, 'renderer, R: Read>(
        &'renderer self,
        stac: &'stac mut Stac<R>,
    ) -> Result<Rendered<'stac, 'renderer, R, Self>, Error> {
        Ok(Rendered {
            handles: stac.handles(),
            renderer: self,
        })
    }

    /// Renders a single object.
    ///
    /// This object will be taken out of the underlying [Stac], and its node's
    /// href will be set to the object's rendered href.
    ///
    /// # Examples
    ///
    /// [BestPracticesRenderer] implements `Render`:
    ///
    /// ```
    /// # use stac::{Render, BestPracticesRenderer, Stac};
    /// let renderer = BestPracticesRenderer::new("data");
    /// let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
    /// let objects = renderer.render_one(&mut stac, handle).unwrap();
    /// ```
    fn render_one<R: Read>(&self, stac: &mut Stac<R>, handle: Handle) -> Result<Object, Error> {
        let is_root = stac.root() == handle;
        let mut object = stac.take(handle)?;
        if is_root {
            object.remove_structural_links();
            let href = self.root_href(&object)?;
            stac.node_mut(handle).href = Some(href.clone());
            object.href = Some(href);
        }
        for child in stac.node(handle).children.clone() {
            stac.ensure_resolved(child)?;
            let href = self.href(
                object.href.as_ref().expect("parents should have hrefs"),
                stac.node(child).object.as_ref().expect("node is resolved"),
            )?;
            let node = stac.node_mut(child);
            node.href = Some(href.clone());
            let child = node.object.as_mut().expect("node is resolved");
            child.remove_structural_links();
            child.href = Some(href);
            let link = self.child_link(&object, child)?;
            object.add_link(link);
            if let Some(link) = self.parent_link(&object, child)? {
                child.add_link(link);
            }
        }
        let root = if is_root {
            &object
        } else {
            stac.node(stac.root())
                .object
                .as_ref()
                .expect("should always have a root object")
        };
        if let Some(link) = self.root_link(root, &object)? {
            object.add_link(link);
        }
        if is_root {
            stac.node_mut(handle).object = Some(object.clone());
        }
        Ok(object)
    }

    /// Gets the root from the root object.
    ///
    /// # Examples
    ///
    /// [BestPracticesRenderer] implements `Render`:
    ///
    /// ```
    /// # use stac::{Render, BestPracticesRenderer, Stac, Item};
    /// let renderer = BestPracticesRenderer::new("a/root/directory");
    /// let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
    /// let href = renderer.root_href(stac.get(handle).unwrap()).unwrap();
    /// assert_eq!(href.as_str(), "a/root/directory/catalog.json");
    /// ```
    fn root_href(&self, root: &Object) -> Result<Href, Error>;

    /// Gets the href from a parent's href to a child.
    ///
    /// # Examples
    ///
    /// [BestPracticesRenderer] implements `Render`:
    ///
    /// ```
    /// # use stac::{Render, BestPracticesRenderer, Href, Stac, Item};
    /// let renderer = BestPracticesRenderer::new("data");
    /// let (mut stac, handle) = Stac::read("data/catalog.json").unwrap();
    /// let item = Item::new("an-id").into();
    /// let href = renderer.href(&Href::new("data/catalog.json"), &item).unwrap();
    /// assert_eq!(href.as_str(), "data/an-id/an-id.json");
    /// ```
    fn href(&self, parent: &Href, child: &Object) -> Result<Href, Error>;

    /// Returns true if this renderer should create absolute hrefs.
    ///
    /// Used for the default link creator provided by the `Render` trait.
    ///
    /// # Examples
    ///
    /// [BestPracticesRenderer] implements `Render`:
    ///
    /// ```
    /// # use stac::{BestPracticesRenderer, Render};
    /// let renderer = BestPracticesRenderer::new("a/root/directory");
    /// assert!(!renderer.is_absolute());
    /// ```
    fn is_absolute(&self) -> bool;

    /// Creates a link from one object to another.
    ///
    /// This default implementation sets the `title` to the target's title, or
    /// its id if the `title` is `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::{BestPracticesRenderer, Link, Render};
    /// let renderer = BestPracticesRenderer::new("a/root/directory");
    /// let link = renderer.link(
    ///     &stac::read("data/catalog.json").unwrap(),
    ///     &stac::read("data/simple-item.json").unwrap(),
    ///     Link::child
    /// ).unwrap();
    /// ```
    fn link<F>(&self, from: &Object, to: &Object, new_link: F) -> Result<Link, Error>
    where
        F: Fn(String) -> Link,
    {
        let to_href = to.href.as_ref().ok_or(Error::MissingHref)?;
        let href = if self.is_absolute() {
            to_href.clone().make_absolute()?
        } else {
            from.href
                .as_ref()
                .ok_or(Error::MissingHref)?
                .make_relative(to_href)
        };
        let mut link = new_link(href.as_str().to_string());
        link.title = to
            .title()
            .map(|title| title.to_string())
            .or_else(|| Some(to.id().to_string()));
        Ok(link)
    }

    /// Creates a child link.
    fn child_link(&self, parent: &Object, child: &Object) -> Result<Link, Error> {
        let link = if child.is_item() {
            self.link(parent, child, Link::item)?
        } else {
            self.link(parent, child, Link::child)?
        };
        Ok(link)
    }

    /// Creates a parent link.
    fn parent_link(&self, parent: &Object, child: &Object) -> Result<Option<Link>, Error> {
        let link = self.link(child, parent, Link::parent)?;
        Ok(Some(link))
    }

    /// Creates a root link.
    fn root_link(&self, root: &Object, child: &Object) -> Result<Option<Link>, Error> {
        let link = self.link(child, root, Link::root)?;
        Ok(Some(link))
    }
}

/// A renderer for creating STAC catalogs according to the [best practices](https://github.com/radiantearth/stac-spec/blob/master/best-practices.md).
#[derive(Debug)]
pub struct BestPracticesRenderer {
    root: Href,
    is_absolute: bool,
}

/// An iterator over objects laid out according to the STAC best practices.
#[derive(Debug)]
pub struct Rendered<'stac, 'renderer, R: Read, T: Render + ?Sized> {
    handles: Handles<'stac, R>,
    renderer: &'renderer T,
}

impl BestPracticesRenderer {
    /// Creates a new best practices renderer rooted at the provided directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::BestPracticesRenderer;
    /// let renderer = BestPracticesRenderer::new("data");
    /// ```
    pub fn new<T>(root: T) -> BestPracticesRenderer
    where
        T: Into<Href>,
    {
        let mut root = root.into();
        root.ensure_ends_in_slash();
        BestPracticesRenderer {
            root,
            is_absolute: false,
        }
    }

    fn file_name(&self, object: &Object) -> String {
        if object.is_catalog() {
            "catalog.json".to_string()
        } else if object.is_collection() {
            "collection.json".to_string()
        } else {
            format!("{}.json", object.id())
        }
    }
}

impl Render for BestPracticesRenderer {
    fn root_href(&self, root: &Object) -> Result<Href, Error> {
        let file_name = self.file_name(root);
        self.root.join(&file_name)
    }

    fn href(&self, parent: &Href, child: &Object) -> Result<Href, Error> {
        let file_name = self.file_name(child);
        parent.join(&format!("{}/{}", child.id(), file_name))
    }

    fn is_absolute(&self) -> bool {
        self.is_absolute
    }
}

impl<R: Read, T: Render> Iterator for Rendered<'_, '_, R, T> {
    type Item = Result<Object, Error>;
    fn next(&mut self) -> Option<Self::Item> {
        self.handles.next().map(|result| {
            result.and_then(|handle| self.renderer.render_one(self.handles.stac, handle))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{BestPracticesRenderer, Render};
    use crate::{Catalog, Stac};

    #[test]
    fn render_one() {
        let renderer = BestPracticesRenderer::new("data");
        let (mut stac, handle) = Stac::with_root(Catalog::new("an-id")).unwrap();
        let object = renderer.render_one(&mut stac, handle).unwrap();
        assert_eq!(object.href.as_ref().unwrap().as_str(), "data/catalog.json");
        for link in object.links() {
            if link.is_root() {
                assert_eq!(link.href, "./catalog.json");
            } else if link.is_parent() {
                panic!("Root should not have a parent link");
            } else if link.is_child() {
                panic!("There are no children on this catalog");
            } else if link.is_item() {
                panic!("There are no items on this catalog");
            } else if link.is_self() {
                assert_eq!(
                    link.href,
                    std::fs::canonicalize("data/catalog.json")
                        .unwrap()
                        .to_string_lossy()
                )
            }
        }
    }
}
