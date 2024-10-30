use crate::{Catalog, Collection, Error, Href, Item, Link, Links, Result, Value};

/// A node in a STAC tree.
#[derive(Debug)]
pub struct Node {
    /// The value of the node.
    pub value: Container,

    /// The child nodes.
    pub children: Vec<Node>,

    /// The node's items.
    pub items: Vec<Item>,
}

/// A STAC container, i.e. a [Catalog] or a [Collection].
#[derive(Debug)]
pub enum Container {
    /// A [Collection].
    Collection(Collection),

    /// A [Catalog].
    Catalog(Catalog),
}

impl Node {
    /// Resolves all child and item links in this node.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Catalog, Node};
    ///
    /// let node: Node = stac::read::<Catalog>("examples/catalog.json").unwrap().into();
    /// node.resolve().unwrap();
    /// ```
    pub fn resolve(&mut self) -> Result<()> {
        let links = std::mem::take(self.value.links_mut());
        let href = self.value.href().map(String::from);
        for mut link in links {
            if link.is_child() {
                link.make_absolute(href.as_deref())?;
                // TODO enable object store
                tracing::debug!("resolving child: {}", link.href);
                let child: Container = crate::read::<Value>(link.href)?.try_into()?;
                self.children.push(child.into());
            } else if link.is_item() {
                link.make_absolute(href.as_deref())?;
                tracing::debug!("resolving item: {}", link.href);
                let item = crate::read::<Item>(link.href)?;
                self.items.push(item);
            } else {
                self.value.links_mut().push(link);
            }
        }
        Ok(())
    }
}

impl From<Catalog> for Node {
    fn from(value: Catalog) -> Self {
        Container::from(value).into()
    }
}

impl From<Catalog> for Container {
    fn from(value: Catalog) -> Self {
        Container::Catalog(value)
    }
}

impl From<Collection> for Node {
    fn from(value: Collection) -> Self {
        Container::from(value).into()
    }
}

impl From<Collection> for Container {
    fn from(value: Collection) -> Self {
        Container::Collection(value)
    }
}

impl From<Container> for Node {
    fn from(value: Container) -> Self {
        Node {
            value,
            children: Vec::new(),
            items: Vec::new(),
        }
    }
}

impl TryFrom<Value> for Container {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Catalog(c) => Ok(c.into()),
            Value::Collection(c) => Ok(c.into()),
            _ => Err(Error::IncorrectType {
                actual: value.type_name().to_string(),
                expected: "Catalog or Collection".to_string(),
            }),
        }
    }
}

impl Links for Container {
    fn links(&self) -> &[Link] {
        match self {
            Container::Catalog(c) => c.links(),
            Container::Collection(c) => c.links(),
        }
    }

    fn links_mut(&mut self) -> &mut Vec<Link> {
        match self {
            Container::Catalog(c) => c.links_mut(),
            Container::Collection(c) => c.links_mut(),
        }
    }
}

impl Href for Container {
    fn href(&self) -> Option<&str> {
        match self {
            Container::Catalog(c) => c.href(),
            Container::Collection(c) => c.href(),
        }
    }

    fn set_href(&mut self, href: impl ToString) {
        match self {
            Container::Catalog(c) => c.set_href(href),
            Container::Collection(c) => c.set_href(href),
        }
    }

    fn clear_href(&mut self) {
        match self {
            Container::Catalog(c) => c.clear_href(),
            Container::Collection(c) => c.clear_href(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Node;
    use crate::{Catalog, Collection, Links};

    #[test]
    fn into_node() {
        let _ = Node::from(Catalog::new("an-id", "a description"));
        let _ = Node::from(Collection::new("an-id", "a description"));
    }

    #[test]
    fn resolve() {
        let mut node: Node = crate::read::<Catalog>("examples/catalog.json")
            .unwrap()
            .into();
        node.resolve().unwrap();
        assert_eq!(node.children.len(), 3);
        assert_eq!(node.items.len(), 1);
        assert_eq!(node.value.links().len(), 2);
    }
}
