//! Input and output (IO) functions.

#[cfg(feature = "geoparquet")]
pub mod geoparquet;
pub mod json;

use crate::{Href, Result};
#[cfg(feature = "reqwest")]
use reqwest::blocking::Response;
use serde::de::DeserializeOwned;
use std::{fs::File, path::Path};
use url::Url;

/// Reads any STAC object from an href.
///
/// If the `geoparquet` feature is enabled, and the href's extension is
/// `geoparquet` or `parquet`, the data will be read as
/// [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet). This is
/// more inefficient than using [crate::io::geoparquet::read], so prefer that if you
/// know your href points to geoparquet data.
///
/// Use [crate::io::json::read] if you want to ensure that your data are read as JSON.
///
/// # Examples
///
/// ```
/// let item: stac::Item = stac::read("examples/simple-item.json").unwrap();
/// ```
pub fn read<T: Href + DeserializeOwned>(href: impl ToString) -> Result<T> {
    let href = href.to_string();
    if crate::geoparquet::has_extension(&href) {
        #[cfg(feature = "geoparquet")]
        {
            serde_json::from_value(serde_json::to_value(geoparquet::read(href)?)?)
                .map_err(crate::Error::from)
        }
        #[cfg(not(feature = "geoparquet"))]
        {
            log::warn!("{} has a geoparquet extension, but this crate was not built with the `geoparquet` feature. Reading as JSON.", href);
            json::read(href)
        }
    } else {
        json::read(href)
    }
}

trait Read<T: Href + DeserializeOwned> {
    fn read(href: impl ToString) -> Result<T> {
        let href = href.to_string();
        let mut value: T = if let Some(url) = crate::href_to_url(&href) {
            Self::read_from_url(url)?
        } else {
            Self::read_from_path(&href)?
        };
        value.set_href(href);
        Ok(value)
    }

    fn read_from_path(path: impl AsRef<Path>) -> Result<T> {
        let file = File::open(path.as_ref())?;
        Self::read_from_file(file)
    }

    fn read_from_file(file: File) -> Result<T>;

    #[cfg(feature = "reqwest")]
    fn read_from_url(url: Url) -> Result<T> {
        let response = reqwest::blocking::get(url.clone())?;
        Self::from_response(response)
    }

    #[cfg(feature = "reqwest")]
    fn from_response(response: Response) -> Result<T>;

    #[cfg(not(feature = "reqwest"))]
    fn read_from_url(_: Url) -> Result<T> {
        Err(crate::Error::ReqwestNotEnabled)
    }
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

    read!(read_item_from_path, "examples/simple-item.json", Item);
    read!(read_catalog_from_path, "examples/catalog.json", Catalog);
    read!(
        read_collection_from_path,
        "examples/collection.json",
        Collection
    );
    read!(
        read_item_collection_from_path,
        "data/item-collection.json",
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

    #[test]
    #[cfg(feature = "geoparquet")]
    fn read_geoparquet() {
        let _: ItemCollection = super::read("data/extended-item.parquet").unwrap();
    }

    #[test]
    #[cfg(not(feature = "geoparquet"))]
    fn read_geoparquet() {
        let _ = super::read::<ItemCollection>("data/extended-item.parquet").unwrap_err();
    }
}
