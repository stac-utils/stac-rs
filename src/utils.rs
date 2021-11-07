//! Utility functions.

use crate::Error;
use std::path::Path;
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
/// assert_eq!(utils::absolute_href("/a/path", None).unwrap(), "/a/path");
/// assert_eq!(utils::absolute_href("/a/path", Some("/does/not/matter")).unwrap(), "/a/path");
/// assert_eq!(utils::absolute_href("a/path", Some("/does/matter")).unwrap(), "/does/matter/a/path");
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
            Ok(Path::new(base).join(href).to_string_lossy().into_owned())
        }
    } else {
        std::fs::canonicalize(href)
            .map(|path_buf| path_buf.to_string_lossy().into_owned())
            .map_err(Error::from)
    }
}
