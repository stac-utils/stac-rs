use crate::{Catalog, Collection, Error, Href, Item, Link, Links, Result, Value};
use std::collections::VecDeque;

/// A node in a STAC tree.
#[derive(Debug)]
pub struct Node {
    /// The value of the node.
    pub value: Container,

    /// The child nodes.
    pub children: VecDeque<Node>,

    /// The node's items.
    pub items: VecDeque<Item>,
}

/// A STAC container, i.e. a [Catalog] or a [Collection].
#[derive(Debug)]
pub enum Container {
    /// A [Collection].
    Collection(Collection),

    /// A [Catalog].
    Catalog(Catalog),
}

/// An iterator over a node and all of its descendants.
#[derive(Debug)]
pub struct IntoValues {
    node: Option<Node>,
    children: VecDeque<Node>,
    items: VecDeque<Item>,
}

impl Node {
    /// Resolves all child and item links in this node.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Catalog, Node};
    ///
    /// let mut node: Node = stac::read::<Catalog>("examples/catalog.json").unwrap().into();
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
                self.children.push_back(child.into());
            } else if link.is_item() {
                link.make_absolute(href.as_deref())?;
                tracing::debug!("resolving item: {}", link.href);
                let item = crate::read::<Item>(link.href)?;
                self.items.push_back(item);
            } else {
                self.value.links_mut().push(link);
            }
        }
        Ok(())
    }

    /// Creates a consuming iterator over this node and its children and items.
    ///
    /// This iterator will visit all children (catalogs and collections) first,
    /// then visit all the items.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Node, Catalog};
    ///
    /// let mut node: Node = Catalog::new("an-id", "a description").into();
    /// node.children
    ///     .push_back(Catalog::new("child", "child catalog").into());
    /// let values: Vec<_> = node.into_values().collect::<Result<_, _>>().unwrap();
    /// assert_eq!(values.len(), 2);
    /// ```
    pub fn into_values(self) -> IntoValues {
        IntoValues {
            node: Some(self),
            children: VecDeque::new(),
            items: VecDeque::new(),
        }
    }
}

impl Iterator for IntoValues {
    type Item = Result<Value>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(mut node) = self.node.take() {
            self.children.append(&mut node.children);
            self.items.append(&mut node.items);
            Some(Ok(node.value.into()))
        } else if let Some(child) = self.children.pop_front() {
            self.node = Some(child);
            self.next()
        } else {
            self.items.pop_front().map(|item| Ok(item.into()))
        }
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
            children: VecDeque::new(),
            items: VecDeque::new(),
        }
    }
}

impl TryFrom<Value> for Container {
    type Error = Error;

    fn try_from(value: Value) -> std::result::Result<Self, Self::Error> {
        match value {
            Value::Catalog(c) => Ok(c.into()),
            Value::Collection(c) => Ok(c.into()),
            _ => Err(stac_types::Error::IncorrectType {
                actual: value.type_name().to_string(),
                expected: "Catalog or Collection".to_string(),
            }
            .into()),
        }
    }
}

impl From<Container> for Value {
    fn from(value: Container) -> Self {
        match value {
            Container::Catalog(c) => Value::Catalog(c),
            Container::Collection(c) => Value::Collection(c),
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
    #[ignore = "skipping while we debug paths"]
    fn resolve() {
        let mut node: Node = crate::read::<Catalog>("examples/catalog.json")
            .unwrap()
            .into();
        node.resolve().unwrap();
        assert_eq!(node.children.len(), 3);
        assert_eq!(node.items.len(), 1);
        assert_eq!(node.value.links().len(), 2);
    }

    #[test]
    fn into_values() {
        let mut node: Node = Catalog::new("an-id", "a description").into();
        node.children
            .push_back(Catalog::new("child", "child catalog").into());
        let mut iter = node.into_values();
        let _root = iter.next().unwrap().unwrap();
        let _child = iter.next().unwrap().unwrap();
        assert!(iter.next().is_none());
    }
}
