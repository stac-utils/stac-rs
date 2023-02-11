use crate::{Item, Result, Search};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use stac::{Link, Links};
use url::Url;

const ITEM_COLLECTION_TYPE: &str = "FeatureCollection";

/// The return value of the `/items` and `/search` endpoints.
///
/// This might be a [stac::ItemCollection], but if the [fields
/// extension](https://github.com/stac-api-extensions/fields) is used, it might
/// not be. Defined by the [itemcollection
/// fragment](https://github.com/radiantearth/stac-api-spec/blob/main/fragments/itemcollection/README.md).
#[derive(Debug, Serialize, Deserialize)]
pub struct ItemCollection {
    /// Always "FeatureCollection" to provide compatibility with GeoJSON.
    pub r#type: String,

    /// A possibly-empty array of Item objects.
    #[serde(rename = "features")]
    pub items: Vec<Item>,

    /// An array of Links related to this ItemCollection.
    pub links: Vec<Link>,

    /// The number of Items that meet the selection parameters, possibly estimated.
    #[serde(skip_serializing_if = "Option::is_none", rename = "numberMatched")]
    pub number_matched: Option<u64>,

    /// The number of Items in the features array.
    #[serde(skip_serializing_if = "Option::is_none", rename = "numberReturned")]
    pub number_returned: Option<u64>,

    /// The search-related metadata for the [ItemCollection].
    ///
    /// Part of the [context extension](https://github.com/stac-api-extensions/context).
    pub context: Option<Context>,

    /// Additional fields.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

/// The search-related metadata for the [ItemCollection].
///
/// Part of the [context extension](https://github.com/stac-api-extensions/context).
#[derive(Debug, Serialize, Deserialize)]
pub struct Context {
    /// The count of results returned by this response. Equal to the cardinality
    /// of features array.
    pub returned: u64,

    /// The maximum number of results to which the result was limited.
    pub limit: Option<u64>,

    /// The count of total number of results that match for this query, possibly
    /// estimated, particularly in the context of NoSQL data stores.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub matched: Option<u64>,

    /// Additional fields.
    #[serde(flatten)]
    pub additional_fields: Map<String, Value>,
}

impl ItemCollection {
    /// Creates a new [ItemCollection] from a vector of items.
    ///
    /// # Examples
    ///
    /// ```
    /// let item: stac_api::Item = stac::Item::new("an-id").try_into().unwrap();
    /// let item_collection = stac_api::ItemCollection::new(vec![item]).unwrap();
    /// ```
    pub fn new(items: Vec<Item>) -> Result<ItemCollection> {
        let number_returned = items.len();
        Ok(ItemCollection {
            r#type: ITEM_COLLECTION_TYPE.to_string(),
            items,
            links: Vec::new(),
            number_matched: None,
            number_returned: Some(number_returned.try_into()?),
            context: None,
            additional_fields: Map::new(),
        })
    }

    /// Returns the next url and search for this item collection.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac_api::ItemCollection;
    /// let value = stac::read_json("examples/planetary-computer-page.geojson").unwrap();
    /// let item_collection: ItemCollection = serde_json::from_value(value).unwrap();
    /// let (url, search) = item_collection.next_url_and_search().unwrap().unwrap();
    /// ```
    pub fn next_url_and_search(&self) -> Result<Option<(Url, Option<Search>)>> {
        if let Some(link) = self.link("next") {
            let url = link.href.parse()?;
            let search = if let Some(body) = link.body.as_ref() {
                serde_json::from_value(Value::Object(body.clone()))?
            } else {
                None
            };
            Ok(Some((url, search)))
        } else {
            Ok(None)
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

#[cfg(test)]
mod tests {
    use super::ItemCollection;

    #[test]
    fn next_search() {
        let value = stac::read_json("examples/planetary-computer-page.geojson").unwrap();
        let item_collection: ItemCollection = serde_json::from_value(value).unwrap();
        let (url, search) = item_collection.next_url_and_search().unwrap().unwrap();
        assert_eq!(
            url.as_str(),
            "https://planetarycomputer.microsoft.com/api/stac/v1/search"
        );
        let search = search.unwrap();
        assert_eq!(
            search.collections.unwrap(),
            vec!["sentinel-2-l2a".to_string()]
        );
        assert_eq!(search.limit.unwrap(), 1);
        assert_eq!(
            search.additional_fields["token"],
            "next:S2A_MSIL2A_20230210T165801_R140_T60CWQ_20230211T071706"
        );
    }
}
