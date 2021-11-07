//! Utility functions.

use crate::Error;
use log::warn;
use std::path::{Path, PathBuf};
use url::Url;

/// Creates an absolute href from an href and an optional base.
///
/// If neither the `href` nor `base` are absolute, the href will be resolved to
/// absolute by using the local filesystem.
///
/// # Examples
///
/// ```
/// use stac::utils;
/// let href = utils::absolute_href("data", None).unwrap();
/// assert!(href.starts_with("/"));
/// assert_eq!(utils::absolute_href("a/path", Some("http://example.com")).unwrap(), "http://example.com/a/path");
/// ```
pub fn absolute_href(href: &str, base: Option<&str>) -> Result<String, Error> {
    if Url::parse(href).is_ok() || Path::new(href).is_absolute() {
        Ok(href.to_string())
    } else if let Some(base) = base {
        if let Ok(base) = Url::parse(base) {
            base.join(href)
                .map(|url| url.to_string())
                .map_err(Error::from)
        } else {
            let mut base_path_buf = PathBuf::from(base);
            if !base_path_buf.pop() {
                warn!("No parent in base url: {}", base);
            }
            let href = base_path_buf.join(href);
            std::fs::canonicalize(href)
                .map(|path_buf| path_buf.to_string_lossy().into_owned())
                .map_err(Error::from)
        }
    } else {
        std::fs::canonicalize(href)
            .map(|path_buf| path_buf.to_string_lossy().into_owned())
            .map_err(Error::from)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn catalog_base() {
        let href = super::absolute_href(
            "./extensions-collection/collection.json",
            Some("data/catalog.json"),
        )
        .unwrap();
        assert!(href.ends_with("/data/extensions-collection/collection.json"))
    }
}
