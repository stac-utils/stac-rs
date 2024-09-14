use crate::{Error, Item, Link, Links, Migrate, Object, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::{io::Write, ops::Deref, vec::IntoIter};

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

impl IntoIterator for ItemCollection {
    type IntoIter = IntoIter<Item>;
    type Item = Item;
    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

impl Deref for ItemCollection {
    type Target = Vec<Item>;
    fn deref(&self) -> &Self::Target {
        &self.items
    }
}

impl Object for ItemCollection {
    const TYPE: &str = ITEM_COLLECTION_TYPE;
    fn href(&self) -> Option<&str> {
        self.href.as_deref()
    }
    fn href_mut(&mut self) -> &mut Option<String> {
        &mut self.href
    }
    #[cfg(feature = "geoparquet")]
    fn geoparquet_from_bytes(bytes: impl Into<bytes::Bytes>) -> Result<Self> {
        crate::io::geoparquet::from_reader(bytes.into())
    }
    #[cfg(feature = "geoparquet")]
    fn geoparquet_from_file(file: std::fs::File) -> Result<Self> {
        crate::io::geoparquet::from_reader(file)
    }
    #[cfg(feature = "geoparquet")]
    fn geoparquet_into_writer(
        self,
        writer: impl Write + Send,
        compression: Option<parquet::basic::Compression>,
    ) -> Result<()> {
        if let Some(compression) = compression {
            crate::io::geoparquet::to_writer_with_compression(writer, self, compression)
        } else {
            crate::io::geoparquet::to_writer(writer, self)
        }
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

impl Migrate for ItemCollection {
    fn migrate(mut self, version: &crate::Version) -> Result<Self> {
        let mut items = Vec::with_capacity(self.items.len());
        for item in self.items {
            items.push(item.migrate(version)?);
        }
        self.items = items;
        Ok(self)
    }
}

impl TryFrom<Value> for ItemCollection {
    type Error = Error;

    fn try_from(value: Value) -> Result<Self> {
        match serde_json::from_value::<ItemCollection>(value.clone()) {
            Ok(item_collection) => Ok(item_collection),
            Err(err) => {
                if let Value::Array(array) = value {
                    let mut items = Vec::new();
                    for item in array {
                        items.push(serde_json::from_value(item)?);
                    }
                    Ok(items.into())
                } else {
                    Err(Error::from(err))
                }
            }
        }
    }
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
