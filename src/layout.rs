use crate::{Error, Handle, Href, HrefObject, Link, Object, Read, Result, Stac};

/// Lay out a [Stac].
///
/// The layout process consists of a couple steps:
///
/// 1. Setting the `next_href` on all [Objects](Object) in the [Stac].
/// 2. Creating "structural" links (`parent`, `child`, `item`, and `root`) between all `Objects`.
#[derive(Debug)]
pub struct Layout {
    root: Href,
}

impl Layout {
    /// Creates a new `Layout`.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Layout;
    /// let layout = Layout::new("the/new/root");
    /// ```
    pub fn new<H>(root: H) -> Layout
    where
        H: Into<Href>,
    {
        Self { root: root.into() }
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
    /// let layout = Layout::new("a/new/root");
    /// layout.layout(&mut stac).unwrap();
    /// assert_eq!(stac.next_href(root).unwrap().as_str(), "a/new/root/catalog.json");
    /// ```
    pub fn layout<R>(&self, stac: &mut Stac<R>) -> Result<()>
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
    /// let layout = Layout::new("a/new/root");
    /// let href_objects = layout.render(stac).collect::<Result<Vec<_>, _>>().unwrap();
    /// assert_eq!(href_objects.len(), 6);
    /// ```
    pub fn render<'a, R>(&'a self, stac: Stac<R>) -> impl Iterator<Item = Result<HrefObject>> + 'a
    where
        R: Read + 'a,
    {
        let root = stac.root();
        stac.into_walk(root, |stac, handle| {
            self.layout_one(stac, handle)?;
            let href = stac
                .next_href(handle)
                .expect("href set during layout")
                .clone();
            let object = stac.take(handle).expect("resolved during layout");
            Ok(HrefObject { href, object })
        })
    }

    fn layout_one<R>(&self, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read,
    {
        // TODO remove structural links
        if handle == stac.root() {
            self.set_href(stac, handle)?;
        }
        let root_link = self.create_link(stac, handle, stac.root(), Link::root)?;
        stac.add_link(handle, root_link)?;
        if let Some(parent) = stac.parent(handle) {
            let parent_link = self.create_link(stac, handle, parent, Link::parent)?;
            stac.add_link(handle, parent_link)?;
        }
        for child in stac.children(handle) {
            self.set_href(stac, child)?;
            let child_link = self.create_link(stac, handle, child, Link::child)?;
            stac.add_link(handle, child_link)?;
        }
        // TODO allow for self hrefs
        Ok(())
    }

    fn set_href<R>(&self, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read,
    {
        // TODO add rebase option
        self.best_practices(stac, handle)
    }

    fn best_practices<R>(&self, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read,
    {
        let mut href = if let Some(parent) = stac.parent(handle) {
            let mut directory = String::from(
                stac.next_href(parent)
                    .ok_or(Error::MissingHref)?
                    .directory(),
            );
            directory.push('/');
            directory.push_str(stac.get(handle)?.id());
            directory
        } else {
            String::from(self.root.as_str())
        };
        href.push('/');
        match stac.get(handle)? {
            Object::Item(item) => href.push_str(&item.id),
            Object::Catalog(_) => href.push_str("catalog"),
            Object::Collection(_) => href.push_str("collection"),
        }
        href.push_str(".json");
        stac.set_next_href(handle, href);
        Ok(())
    }

    fn create_link<R, F>(&self, stac: &mut Stac<R>, from: Handle, to: Handle, f: F) -> Result<Link>
    where
        R: Read,
        F: Fn(String) -> Link,
    {
        let from_href = stac.next_href(from).ok_or(Error::MissingHref)?;
        let to_href = stac.next_href(to).ok_or(Error::MissingHref)?;
        // TODO allow for absolute hrefs
        let href = from_href.make_relative(to_href.clone());
        let mut link = f(href.into());
        link.title = stac.get(to)?.title().map(String::from);
        Ok(link)
    }
}

#[cfg(test)]
mod tests {
    use super::Layout;
    use crate::{Catalog, Collection, Item, Stac};

    #[test]
    fn layout_best_practices() {
        let catalog = Catalog::new("root");
        let (mut stac, root) = Stac::new(catalog).unwrap();
        let collection = stac
            .add_child(root, Collection::new("child-collection"))
            .unwrap();
        let item = stac.add_child(collection, Item::new("an-item")).unwrap();
        let layout = Layout::new("stac/root");
        let _ = layout.layout(&mut stac).unwrap();

        assert_eq!(
            stac.next_href(root).unwrap().as_str(),
            "stac/root/catalog.json"
        );
        let root = stac.get(root).unwrap();
        assert_eq!(root.root_link().as_ref().unwrap().href, "./catalog.json");
        assert!(root.parent_link().is_none());
        let child_links: Vec<_> = root.child_links().collect();
        assert_eq!(child_links.len(), 1);
        let child_link = child_links[0];
        assert_eq!(child_link.href, "./child-collection/collection.json");

        assert_eq!(
            stac.next_href(collection).unwrap().as_str(),
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
            stac.next_href(item).unwrap().as_str(),
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

    // TODO test that linking removes previous structura

    #[test]
    fn render_spec() {
        let (stac, _) = Stac::read("data/catalog.json").unwrap();
        let layout = Layout::new("a/new/root");
        let href_objects = layout.render(stac).collect::<Result<Vec<_>, _>>().unwrap();
        assert_eq!(href_objects.len(), 6);
    }
}
