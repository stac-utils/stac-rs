use crate::{Error, Href, Result, Value};
use std::{fs::File, io::BufReader, path::Path};
use url::Url;

/// Reads any STAC object from an href.
///
/// # Examples
///
/// ```
/// let item = stac::read("data/simple-item.json").unwrap();
/// assert!(item.is_item());
/// ```
pub fn read(href: impl ToString) -> Result<Value> {
    let href = href.to_string();
    let value = read_json(&href)?;
    let mut value = Value::from_json(value)?;
    value.set_href(href);
    Ok(value)
}

/// Reads any JSON value from an href.
///
/// # Examples
///
/// ```
/// let value = stac::read_json("data/simple-item.json").unwrap();
/// ```
pub fn read_json(href: &str) -> Result<serde_json::Value> {
    if let Ok(url) = Url::parse(&href) {
        read_json_from_url(url)
    } else {
        read_json_from_path(href)
    }
}

fn read_json_from_path<P: AsRef<Path>>(path: P) -> Result<serde_json::Value> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    serde_json::from_reader(reader).map_err(Error::from)
}

#[cfg(feature = "reqwest")]
fn read_json_from_url(url: Url) -> Result<serde_json::Value> {
    let response = reqwest::blocking::get(url.clone())?;
    response.json().map_err(Error::from)
}

#[cfg(not(feature = "reqwest"))]
fn read_json_from_url(_: Url) -> Result<serde_json::Value> {
    Err(crate::Error::ReqwestNotEnabled)
}

#[cfg(test)]
mod tests {
    use crate::Value;

    macro_rules! read {
        ($function:ident, $filename:expr, $value:pat) => {
            #[test]
            fn $function() {
                use crate::Href;

                let value = crate::read($filename).unwrap();
                assert!(matches!(value, $value));
                assert!(value.href().is_some());
            }
        };
    }

    read!(read_item_from_path, "data/simple-item.json", Value::Item(_));
    read!(
        read_catalog_from_path,
        "data/catalog.json",
        Value::Catalog(_)
    );
    read!(
        read_collection_from_path,
        "data/collection.json",
        Value::Collection(_)
    );
    read!(
        read_item_collection_from_path,
        "examples/item-collection.json",
        Value::ItemCollection(_)
    );

    #[cfg(feature = "reqwest")]
    mod with_reqwest {
        use crate::Value;

        read!(
            read_item_from_url,
            "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json",
            Value::Item(_)
        );
        read!(
            read_catalog_from_url,
            "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/catalog.json",
            Value::Catalog(_)
        );
        read!(
            read_collection_from_url,
            "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/collection.json",
            Value::Collection(_)
        );
    }

    #[cfg(not(feature = "reqwest"))]
    mod without_reqwest {
        #[test]
        fn read_url() {
            assert!(matches!(
                crate::read("http://stac-rs.test/item.json").unwrap_err(),
                crate::Error::ReqwestNotEnabled
            ));
        }
    }
}
