use crate::{Href, Item, Link, Links, Migrate, Result, Version};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};

/// The type field for [ItemCollections](ItemCollection).
pub const ITEM_COLLECTION_TYPE: &str = "FeatureCollection";

/// A [GeoJSON FeatureCollection](https://www.rfc-editor.org/rfc/rfc7946#page-12) of items.
///
/// While not part of the STAC specification, ItemCollections are often used to store many items in a single file.
#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ItemCollection {
    /// The list of [Items](Item).
    ///
    /// The attribute is actually "features", but we rename to "items".
    #[serde(rename = "features")]
    pub items: Vec<Item>,

    /// List of link objects to resources and related URLs.
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub links: Vec<Link>,

    /// Additional fields.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,

    /// The type field.
    ///
    /// Must be set to "FeatureCollection".
    #[serde(
        deserialize_with = "deserialize_type",
        serialize_with = "serialize_type"
    )]
    r#type: String,

    #[serde(skip)]
    href: Option<String>,
}

impl ItemCollection {
    /// Migrates this item collection to another version.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::ItemCollection;
    ///
    /// let mut item_collection: ItemCollection = vec![stac::read("data/simple-item.json").unwrap()].into();
    /// let version = "1.1.0-beta.1".parse().unwrap();
    /// item_collection.migrate(version).unwrap();
    /// ```
    pub fn migrate(&mut self, version: Version) -> Result<()> {
        for item in self.items.iter_mut() {
            item.migrate(version)?;
        }
        Ok(())
    }
}

impl From<Vec<Item>> for ItemCollection {
    fn from(items: Vec<Item>) -> Self {
        ItemCollection {
            r#type: ITEM_COLLECTION_TYPE.to_string(),
            items,
            links: Vec::new(),
            additional_fields: Map::new(),
            href: None,
        }
    }
}

impl FromIterator<Item> for ItemCollection {
    fn from_iter<I: IntoIterator<Item = Item>>(iter: I) -> Self {
        iter.into_iter().collect::<Vec<_>>().into()
    }
}

impl Href for ItemCollection {
    fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }

    fn set_href(&mut self, href: impl ToString) {
        self.href = Some(href.to_string())
    }
}

impl Links for ItemCollection {
    fn links(&self) -> &[Link] {
        &self.links
    }
    fn links_mut(&mut self) -> &mut Vec<Link> {
        &mut self.links
    }
}

fn deserialize_type<'de, D>(deserializer: D) -> std::result::Result<String, D::Error>
where
    D: serde::de::Deserializer<'de>,
{
    crate::deserialize_type(deserializer, ITEM_COLLECTION_TYPE)
}

fn serialize_type<S>(r#type: &String, serializer: S) -> std::result::Result<S::Ok, S::Error>
where
    S: serde::ser::Serializer,
{
    crate::serialize_type(r#type, serializer, ITEM_COLLECTION_TYPE)
}

#[cfg(test)]
mod tests {
    use super::ItemCollection;
    use crate::Item;

    #[test]
    fn item_collection_from_vec() {
        let items = vec![Item::new("a"), Item::new("b")];
        let _ = ItemCollection::from(items);
    }

    #[test]
    fn item_collection_from_iter() {
        let items = vec![Item::new("a"), Item::new("b")];
        let _ = ItemCollection::from_iter(items.into_iter());
    }
}
