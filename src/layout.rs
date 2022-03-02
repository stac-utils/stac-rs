use crate::{Error, Handle, Href, HrefObject, Link, Object, Read, Result, Stac};

/// Lay out a [Stac].
///
/// The layout process consists of a couple steps:
///
/// 1. Setting the `next_href` on all [Objects](Object) in the [Stac].
/// 2. Creating "structural" links (`parent`, `child`, `item`, and `root`) between all `Objects`.
#[derive(Debug)]
pub struct Layout<S: Strategy> {
    root: Href,
    strategy: S,
}

/// Sets the [Href] for [Objects](Object) in a [Stac].
///
/// You can implement your own layout structure by implementing `Strategy`.
///
/// # Examples
///
/// [Rebase] implements `Strategy`:
///
/// ```
/// use stac::{Layout, Rebase};
/// let layout = Layout::new("a/new/root").with_strategy(Rebase::default());
/// ```
pub trait Strategy {
    /// Sets the [Href] for an [Object] in a [Stac].
    ///
    /// Takes a mutable reference because some strategies might need to save information about the state of the [Stac], e.g. the original root href.
    ///
    /// # Examples
    ///
    /// [BestPractices] implements [Strategy]:
    ///
    /// ```
    /// use stac::{Href, Catalog, Item, Stac, BestPractices, Strategy};
    /// let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
    /// let item = stac.add_child(root, Item::new("an-item")).unwrap();
    /// let root_href = Href::new("new/root/");
    /// let mut best_practices = BestPractices;
    /// best_practices.set_href(&root_href, &mut stac, root).unwrap();
    /// assert_eq!(stac.href(root).unwrap().as_str(), "new/root/catalog.json");
    /// best_practices.set_href(&root_href, &mut stac, item).unwrap();
    /// assert_eq!(stac.href(item).unwrap().as_str(), "new/root/an-item/an-item.json");
    /// ```
    fn set_href<R>(&mut self, root: &Href, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read;
}

/// Sets [Hrefs](Href) according to the STAC [best practices](https://github.com/radiantearth/stac-spec/blob/master/best-practices.md#catalog-layout).
///
/// # Examples
///
/// ```
/// use stac::{Stac, Catalog, Href, BestPractices, Strategy};
/// let (mut stac, root) = Stac::new(Catalog::new("root")).unwrap();
/// let root_href = Href::new("a/new/root/");
/// let mut best_practices = BestPractices;
/// best_practices.set_href(&root_href, &mut stac, root).unwrap();
/// assert_eq!(stac.href(root).unwrap().as_str(), "a/new/root/catalog.json");
/// ```
#[derive(Debug)]
pub struct BestPractices;

/// Returns a next [Hrefs](Href) that moves objects from one root directory to another.
///
/// # Examples
///
/// ```
/// use stac::{Stac, Catalog, Href, Rebase, HrefObject, Strategy};
/// let (mut stac, root) = Stac::new(HrefObject::new(Catalog::new("root"), "old/path/catalog.json")).unwrap();
/// let root_href = Href::new("a/new/root/");
/// let mut rebase = Rebase::default();
/// rebase.set_href(&root_href, &mut stac, root).unwrap();
/// assert_eq!(stac.href(root).unwrap().as_str(), "a/new/root/catalog.json");
/// ```
#[derive(Debug, Default)]
pub struct Rebase {
    old_root: Option<Href>,
}

impl Layout<BestPractices> {
    /// Creates a new `Layout`.
    ///
    /// The root should be a directory, not a file name.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Layout;
    /// let layout = Layout::new("the/new/root");
    /// ```
    pub fn new<H>(root: H) -> Layout<BestPractices>
    where
        H: Into<Href>,
    {
        let mut root = root.into();
        root.ensure_ends_in_slash();
        Self {
            root: root.into(),
            strategy: BestPractices,
        }
    }
}

impl<S: Strategy> Layout<S> {
    /// Changes how [Hrefs](Href) are set.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Layout, Rebase};
    /// let layout = Layout::new("a/new/root").with_strategy(Rebase::default());
    /// ```
    pub fn with_strategy<T>(self, strategy: T) -> Layout<T>
    where
        T: Strategy,
    {
        Layout {
            root: self.root,
            strategy,
        }
    }

    /// Lays out a [Stac].
    ///
    /// Note that this function will load the entire STAC catalog into memory.
    /// If you want to walk over the laid-out objects iteratively, use [render](Layout::render).
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Stac, Layout};
    /// let (mut stac, root) = Stac::read("data/catalog.json").unwrap();
    /// let mut layout = Layout::new("a/new/root");
    /// layout.layout(&mut stac).unwrap();
    /// assert_eq!(stac.href(root).unwrap().as_str(), "a/new/root/catalog.json");
    /// ```
    pub fn layout<R>(&mut self, stac: &mut Stac<R>) -> Result<()>
    where
        R: Read,
    {
        for result in stac.walk(stac.root(), |stac, handle| self.layout_one(stac, handle)) {
            let _ = result?;
        }
        Ok(())
    }

    /// Renders a [Stac], consuming it.
    ///
    /// This returns an iterator over the laid-out [HrefObjects](HrefObject) in a [Stac].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Stac, Layout};
    /// let (mut stac, _) = Stac::read("data/catalog.json").unwrap();
    /// let mut layout = Layout::new("a/new/root");
    /// let href_objects = layout.render(stac).collect::<Result<Vec<_>, _>>().unwrap();
    /// assert_eq!(href_objects.len(), 6);
    /// ```
    pub fn render<'a, R>(
        &'a mut self,
        stac: Stac<R>,
    ) -> impl Iterator<Item = Result<HrefObject>> + 'a
    where
        R: Read + 'a,
    {
        let root = stac.root();
        stac.into_walk(root, |stac, handle| {
            self.layout_one(stac, handle)?;
            let (href, object) = if handle == stac.root() {
                (
                    stac.href(handle).expect("href set during layout").clone(),
                    stac.get(handle).expect("resolved during layout").clone(),
                )
            } else {
                (
                    stac.take_href(handle).expect("href set during layout"),
                    stac.take(handle).expect("resolved during layout"),
                )
            };
            Ok(HrefObject { href, object })
        })
    }

    fn layout_one<R>(&mut self, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read,
    {
        if handle == stac.root() {
            stac.remove_structural_links(handle)?;
            self.set_href(stac, handle)?;
            let root_link = self.create_link(stac, handle, handle, Link::root)?;
            stac.add_link(handle, root_link)?;
        }
        for child in stac.children(handle) {
            stac.remove_structural_links(child)?;
            self.set_href(stac, child)?;
            let child_link = self.create_link(stac, handle, child, Link::child)?;
            stac.add_link(handle, child_link)?;
            let root_link = self.create_link(stac, child, stac.root(), Link::root)?;
            stac.add_link(child, root_link)?;
            let parent_link = self.create_link(stac, child, handle, Link::parent)?;
            stac.add_link(child, parent_link)?;
        }
        // TODO allow for self hrefs
        Ok(())
    }

    fn set_href<R>(&mut self, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read,
    {
        self.strategy.set_href(&self.root, stac, handle)
    }

    fn create_link<R, F>(
        &self,
        stac: &mut Stac<R>,
        from: Handle,
        to: Handle,
        mut f: F,
    ) -> Result<Link>
    where
        R: Read,
        F: FnMut(String) -> Link,
    {
        let from_href = stac.href(from).ok_or(Error::MissingHref)?;
        let to_href = stac.href(to).ok_or(Error::MissingHref)?;
        // TODO allow for absolute hrefs
        let href = from_href.make_relative(to_href.clone());
        let mut link = f(href.into());
        link.title = stac.get(to)?.title().map(String::from);
        Ok(link)
    }
}

impl Strategy for BestPractices {
    fn set_href<R>(&mut self, root: &Href, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read,
    {
        let mut href = if let Some(parent) = stac.parent(handle) {
            let mut directory =
                String::from(stac.href(parent).ok_or(Error::MissingHref)?.directory());
            directory.push('/');
            directory.push_str(stac.get(handle)?.id());
            directory.push('/');
            directory
        } else {
            String::from(root.as_str())
        };
        match stac.get(handle)? {
            Object::Item(item) => href.push_str(&item.id),
            Object::Catalog(_) => href.push_str("catalog"),
            Object::Collection(_) => href.push_str("collection"),
        }
        href.push_str(".json");
        Ok(stac.set_href(handle, href))
    }
}

impl Strategy for Rebase {
    fn set_href<R>(&mut self, root: &Href, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read,
    {
        if handle == stac.root() {
            let old_root = stac.take_href(handle).ok_or(Error::MissingHref)?;
            self.old_root = Some(old_root.clone());
            Ok(stac.set_href(
                handle,
                root.join(self.old_root.as_ref().unwrap().file_name())?,
            ))
        } else {
            let mut href = stac.href(handle).ok_or(Error::MissingHref)?.clone();
            let root_href = self.old_root.as_ref().ok_or(Error::MissingHref)?;
            href.rebase(root_href, root)?;
            Ok(stac.set_href(handle, href))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{Layout, Rebase};
    use crate::{Catalog, Collection, HrefObject, Item, Link, Stac};

    #[test]
    fn layout_best_practices() {
        let catalog = Catalog::new("root");
        let (mut stac, root) = Stac::new(catalog).unwrap();
        let collection = stac
            .add_child(root, Collection::new("child-collection"))
            .unwrap();
        let item = stac.add_child(collection, Item::new("an-item")).unwrap();
        let mut layout = Layout::new("stac/root");
        let _ = layout.layout(&mut stac).unwrap();

        assert_eq!(stac.href(root).unwrap().as_str(), "stac/root/catalog.json");
        let root = stac.get(root).unwrap();
        assert_eq!(root.root_link().as_ref().unwrap().href, "./catalog.json");
        assert!(root.parent_link().is_none());
        let child_links: Vec<_> = root.child_links().collect();
        assert_eq!(child_links.len(), 1);
        let child_link = child_links[0];
        assert_eq!(child_link.href, "./child-collection/collection.json");

        assert_eq!(
            stac.href(collection).unwrap().as_str(),
            "stac/root/child-collection/collection.json"
        );
        let collection = stac.get(collection).unwrap();
        assert_eq!(
            collection.root_link().as_ref().unwrap().href,
            "../catalog.json"
        );
        assert_eq!(
            collection.parent_link().as_ref().unwrap().href,
            "../catalog.json"
        );
        let child_links: Vec<_> = collection.child_links().collect();
        assert_eq!(child_links.len(), 1);
        let child_link = child_links[0];
        assert_eq!(child_link.href, "./an-item/an-item.json");

        assert_eq!(
            stac.href(item).unwrap().as_str(),
            "stac/root/child-collection/an-item/an-item.json"
        );
        let item = stac.get(item).unwrap();
        assert_eq!(
            item.root_link().as_ref().unwrap().href,
            "../../catalog.json"
        );
        assert_eq!(
            item.parent_link().as_ref().unwrap().href,
            "../collection.json"
        );
        assert_eq!(item.child_links().count(), 0);
    }

    #[test]
    fn remove_previous_structural() {
        let mut catalog = Catalog::new("root");
        catalog.links.push(Link::child("data/simple-item.json"));
        let (mut stac, root) = Stac::new(catalog).unwrap();
        let children = stac.children(root);
        assert_eq!(children.len(), 1);
        let _ = stac.remove(children[0]);
        let mut layout = Layout::new("stac/v0");
        layout.layout(&mut stac).unwrap();
        let root = stac.get(root).unwrap();
        assert_eq!(root.child_links().count(), 0);
    }

    #[test]
    fn render_spec() {
        let (stac, _) = Stac::read("data/catalog.json").unwrap();
        let mut layout = Layout::new("a/new/root");
        let href_objects = layout.render(stac).collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(href_objects.len(), 6);
    }

    #[test]
    fn rebase() {
        let catalog = HrefObject::new(Catalog::new("root"), "old/path/catalog.json");
        let (mut stac, root) = Stac::new(catalog).unwrap();
        let item = stac
            .add_child(
                root,
                HrefObject::new(
                    Item::new("an-item"),
                    "old/path/many/sub/dirs/weird-item-name.json",
                ),
            )
            .unwrap();
        let mut layout = Layout::new("the/new/root").with_strategy(Rebase::default());
        layout.layout(&mut stac).unwrap();
        assert_eq!(
            stac.href(root).unwrap().as_str(),
            "the/new/root/catalog.json"
        );
        assert_eq!(
            stac.href(item).unwrap().as_str(),
            "the/new/root/many/sub/dirs/weird-item-name.json"
        );
    }
}
