use crate::{Error, Href, Result};
use serde::de::DeserializeOwned;
use std::{fs::File, io::BufReader, path::Path};
use url::Url;

/// Reads any STAC object from an href.
///
/// # Examples
///
/// ```
/// let item: stac::Item = stac::read("data/simple-item.json").unwrap();
/// ```
pub fn read<T: Href + DeserializeOwned>(href: impl ToString) -> Result<T> {
    let href = href.to_string();
    let mut value: T = read_json(&href)?;
    value.set_href(href);
    Ok(value)
}

pub(crate) fn read_json<T>(href: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    if let Some(url) = crate::href_to_url(href) {
        read_json_from_url(url)
    } else {
        read_json_from_path(href)
    }
}

fn read_json_from_path<T>(path: impl AsRef<Path>) -> Result<T>
where
    T: DeserializeOwned,
{
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).map_err(Error::from)
}

#[cfg(feature = "reqwest")]
fn read_json_from_url<T>(url: Url) -> Result<T>
where
    T: DeserializeOwned,
{
    let response = reqwest::blocking::get(url.clone())?;
    response.json().map_err(Error::from)
}

#[cfg(not(feature = "reqwest"))]
fn read_json_from_url<T>(_: Url) -> Result<T>
where
    T: DeserializeOwned,
{
    Err(Error::ReqwestNotEnabled)
}

#[cfg(test)]
mod tests {
    use crate::{Catalog, Collection, Item, ItemCollection};

    macro_rules! read {
        ($function:ident, $filename:expr, $value:ty) => {
            #[test]
            fn $function() {
                use crate::Href;

                let value: $value = crate::read($filename).unwrap();
                assert!(value.href().is_some());
            }
        };
    }

    read!(read_item_from_path, "data/simple-item.json", Item);
    read!(read_catalog_from_path, "data/catalog.json", Catalog);
    read!(
        read_collection_from_path,
        "data/collection.json",
        Collection
    );
    read!(
        read_item_collection_from_path,
        "examples/item-collection.json",
        ItemCollection
    );

    #[cfg(feature = "reqwest")]
    mod with_reqwest {
        use crate::{Catalog, Collection, Item};

        read!(
            read_item_from_url,
            "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json",
            Item
        );
        read!(
            read_catalog_from_url,
            "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/catalog.json",
            Catalog
        );
        read!(
            read_collection_from_url,
            "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/collection.json",
            Collection
        );
    }

    #[cfg(not(feature = "reqwest"))]
    mod without_reqwest {
        #[test]
        fn read_url() {
            assert!(matches!(
                crate::read::<crate::Item>("http://stac-rs.test/item.json").unwrap_err(),
                crate::Error::ReqwestNotEnabled
            ));
        }
    }
}
