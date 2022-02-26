use crate::{Error, Handle, Href, Object, Read, Result, Stac};

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
            self.best_practices(stac, handle)
        })
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
}
