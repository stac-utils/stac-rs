use crate::{Href, Result, Value};
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
    if let Ok(url) = Url::parse(&href) {
        read_from_url(url)
    } else {
        read_from_path(href)
    }
}

/// Reads any STAC object from a path.
///
/// # Examples
///
/// ```
/// let item = stac::read_from_path("data/simple-item.json").unwrap();
/// assert!(item.is_item());
/// ```
pub fn read_from_path<P: AsRef<Path>>(path: P) -> Result<Value> {
    let file = File::open(path.as_ref())?;
    let reader = BufReader::new(file);
    let value: serde_json::Value = serde_json::from_reader(reader)?;
    let mut value = Value::from_json(value)?;
    value.set_href(path.as_ref().to_string_lossy());
    Ok(value)
}

/// Reads any STAC object from a url.
///
/// # Examples
///
/// ```no_run
/// let url = "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json";
/// let item = stac::read_from_url(url.parse().unwrap()).unwrap();
/// assert!(item.is_item());
/// ```
#[cfg(feature = "reqwest")]
pub fn read_from_url(url: Url) -> Result<Value> {
    let response = reqwest::blocking::get(url.clone())?;
    let value: serde_json::Value = response.json()?;
    let mut value = Value::from_json(value)?;
    value.set_href(url.to_string());
    Ok(value)
}

/// Reads any STAC object from a url.
///
/// Returns an error because reqwest is not enabled.
///
/// # Examples
///
/// ```
/// let url = "http://stac-rs.test/item.json";
/// let err = stac::read_from_url(url.parse().unwrap()).unwrap_err();
/// assert!(matches!(err, stac::Error::ReqwestNotEnabled));
/// ```
#[cfg(not(feature = "reqwest"))]
pub fn read_from_url(_: Url) -> Result<Value> {
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
