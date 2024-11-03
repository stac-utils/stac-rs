use crate::{Container, Links, Node, Result, SelfHref, Value};
use std::{future::Future, pin::Pin};
use tokio::task::JoinSet;
use url::Url;

/// An object that uses object store to resolve links.
#[derive(Debug, Default)]
#[cfg(feature = "object-store")]
pub struct Resolver {
    recursive: bool,
    use_items_endpoint: bool,
}

impl Resolver {
    /// Resolves the links of a node.
    pub fn resolve(&self, mut node: Node) -> Pin<Box<impl Future<Output = Result<Node>> + '_>> {
        Box::pin(async {
            let links = std::mem::take(node.value.links_mut());
            let href = node.value.self_href().cloned();
            let mut join_set = JoinSet::new();
            for mut link in links {
                if link.is_child() {
                    if let Some(href) = &href {
                        link.make_absolute(href)?;
                    }
                    let _ = join_set
                        .spawn(async move { (crate::io::get::<Value>(link.href).await, true) });
                } else if !self.use_items_endpoint && link.is_item() {
                    if let Some(href) = &href {
                        link.make_absolute(href)?;
                    }
                    let _ = join_set.spawn(async move { (crate::io::get(link.href).await, false) });
                } else if self.use_items_endpoint && link.rel == "items" {
                    let mut url: Url = link.href.try_into()?;
                    // TODO make this configurable
                    let _ = url
                        .query_pairs_mut()
                        .append_pair("limit", "1")
                        .append_pair("sortby", "-properties.datetime");
                    let _ = join_set.spawn(async move { (crate::io::get(url).await, false) });
                } else {
                    node.value.links_mut().push(link);
                }
            }
            while let Some(result) = join_set.join_next().await {
                let (result, is_child) = result?;
                let value = result?;
                if is_child {
                    let child = Container::try_from(value)?.into();
                    node.children.push_back(child);
                } else if let Value::ItemCollection(item_collection) = value {
                    node.items.extend(item_collection.into_iter());
                } else {
                    node.items.push_back(value.try_into()?);
                }
            }
            if self.recursive {
                let children = std::mem::take(&mut node.children);
                for child in children {
                    node.children.push_back(self.resolve(child).await?);
                }
            }
            Ok(node)
        })
    }
}
