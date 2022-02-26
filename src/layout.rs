use crate::{Error, Handle, Href, Link, Object, Read, Result, Stac};

/// Lay out a [Stac].
#[derive(Debug)]
pub struct Layout {
    root: Href,
}

impl Layout {
    /// Creates a new `Layout`.
    pub fn new<H>(root: H) -> Layout
    where
        H: Into<Href>,
    {
        Self { root: root.into() }
    }

    /// Sets the hrefs of a [Stac].
    pub fn set_hrefs<R>(&self, stac: &mut Stac<R>) -> Result<()>
    where
        R: Read,
    {
        stac.walk(stac.root(), |stac, handle| {
            // TODO add rebase option
            self.best_practices(stac, handle)
        })
        .collect()
    }

    /// Links.
    pub fn link<R>(&self, stac: &mut Stac<R>) -> Result<()>
    where
        R: Read,
    {
        stac.walk(stac.root(), |stac, handle| self.link_one(stac, handle))
            .collect()
    }

    fn best_practices<R>(&self, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read,
    {
        let mut href = if let Some(parent) = stac.parent(handle) {
            let mut directory =
                String::from(stac.href(parent).ok_or(Error::MissingHref)?.directory());
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
        stac.set_href(handle, href);
        Ok(())
    }

    fn link_one<R>(&self, stac: &mut Stac<R>, handle: Handle) -> Result<()>
    where
        R: Read,
    {
        let root_link = self.create_link(stac, handle, stac.root(), Link::root)?;
        stac.add_link(handle, root_link)?;
        if let Some(parent) = stac.parent(handle) {
            let parent_link = self.create_link(stac, handle, parent, Link::parent)?;
            stac.add_link(handle, parent_link)?;
        }
        for child in stac.children(handle) {
            let child_link = self.create_link(stac, handle, child, Link::child)?;
            stac.add_link(handle, child_link)?;
        }
        Ok(())
    }

    fn create_link<R, F>(&self, stac: &mut Stac<R>, from: Handle, to: Handle, f: F) -> Result<Link>
    where
        R: Read,
        F: Fn(String) -> Link,
    {
        let from_href = stac.href(from).ok_or(Error::MissingHref)?;
        let to_href = stac.href(to).ok_or(Error::MissingHref)?;
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
    fn best_practices() {
        let catalog = Catalog::new("root");
        let (mut stac, root) = Stac::new(catalog).unwrap();
        let collection = stac
            .add_child(root, Collection::new("child-collection"))
            .unwrap();
        let item = stac.add_child(collection, Item::new("an-item")).unwrap();
        let layout = Layout::new("stac/root");
        layout.set_hrefs(&mut stac).unwrap();
        assert_eq!(stac.href(root).unwrap().as_str(), "stac/root/catalog.json");
        assert_eq!(
            stac.href(collection).unwrap().as_str(),
            "stac/root/child-collection/collection.json"
        );
        assert_eq!(
            stac.href(item).unwrap().as_str(),
            "stac/root/child-collection/an-item/an-item.json"
        );
    }

    #[test]
    fn link() {
        let catalog = Catalog::new("root");
        let (mut stac, root) = Stac::new(catalog).unwrap();
        let collection = stac
            .add_child(root, Collection::new("child-collection"))
            .unwrap();
        let item = stac.add_child(collection, Item::new("an-item")).unwrap();
        let layout = Layout::new("stac/root");
        layout.set_hrefs(&mut stac).unwrap();
        layout.link(&mut stac).unwrap();

        let root = stac.get(root).unwrap();
        assert_eq!(root.root_link().as_ref().unwrap().href, "./catalog.json");
        assert!(root.parent_link().is_none());
        let child_links: Vec<_> = root.child_links().collect();
        assert_eq!(child_links.len(), 1);
        let child_link = child_links[0];
        assert_eq!(child_link.href, "./child-collection/collection.json");

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
}
