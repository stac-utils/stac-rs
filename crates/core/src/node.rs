use crate::{Catalog, Collection, Error, Href, Item, Link, Links, Result, SelfHref, Value};
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

/// A resolver that uses object store.
impl Node {
    /// Resolves all child and item links in this node.
    ///
    /// This method uses [crate::Resolver] to resolve links.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::{Catalog, Node};
    ///
    /// let mut node: Node = stac::read::<Catalog>("examples/catalog.json").unwrap().into();
    /// # tokio_test::block_on(async {
    /// let node = node.resolve().await.unwrap();
    /// });
    /// ```
    #[cfg(feature = "object-store")]
    pub async fn resolve(self) -> Result<Node> {
        let resolver = crate::Resolver::default();
        resolver.resolve(self).await
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
            _ => Err(Error::IncorrectType {
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

impl SelfHref for Container {
    fn self_href(&self) -> Option<&Href> {
        match self {
            Container::Catalog(c) => c.self_href(),
            Container::Collection(c) => c.self_href(),
        }
    }

    fn self_href_mut(&mut self) -> &mut Option<Href> {
        match self {
            Container::Catalog(c) => c.self_href_mut(),
            Container::Collection(c) => c.self_href_mut(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Node;
    use crate::{Catalog, Collection};

    #[test]
    fn into_node() {
        let _ = Node::from(Catalog::new("an-id", "a description"));
        let _ = Node::from(Collection::new("an-id", "a description"));
    }

    #[tokio::test]
    #[cfg(feature = "object-store")]
    async fn resolve() {
        use crate::Links;

        let node: Node = crate::read::<Catalog>("examples/catalog.json")
            .unwrap()
            .into();
        let node = node.resolve().await.unwrap();
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
