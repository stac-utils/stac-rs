use crate::Error;
use log::warn;
use path_slash::PathBufExt;
use std::path::{Path, PathBuf};
use url::Url;

/// A wrapper around a parsed href.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Href {
    /// An absolute filesystem path.
    Path(PathBuf),

    /// An absolute URL.
    Url(Url),
}

impl Href {
    /// Creates an absolute href from an href and an optional base.
    ///
    /// If neither the `href` nor `base` are absolute, the href will be resolved to
    /// absolute by using the local filesystem.
    /// 
    /// Note that the href should _always_ be UNIX-style, i.e. with `/` separators.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("data", None).unwrap();
    /// assert!(href.as_path().unwrap().is_absolute());
    /// assert_eq!(
    ///     Href::new("a/path", "http://example.com").unwrap().to_string(),
    ///     "http://example.com/a/path"
    /// );
    /// ```
    #[allow(single_use_lifetimes)] // https://github.com/rust-lang/rust/issues/60554
    pub fn new<'a, T: Into<Option<&'a str>>>(href: &str, base: T) -> Result<Href, Error> {
        let base = base.into();
        if let Ok(url) = Url::parse(href) {
            return Ok(Href::Url(url));
        }
        let href_as_path = PathBuf::from_slash(href);
        if href_as_path.is_absolute() {
            return Ok(Href::Path(href_as_path.to_path_buf()));
        }
        if let Some(base) = base {
            if let Ok(base) = Url::parse(base) {
                base.join(href).map(Href::Url).map_err(Error::from)
            } else {
                let mut base_path_buf = PathBuf::from_slash(base);
                if !base_path_buf.pop() {
                    warn!("No parent in base url: {}", base);
                }
                let href = base_path_buf.join(href_as_path);
                std::fs::canonicalize(href)
                    .map(Href::Path)
                    .map_err(Error::from)
            }
        } else {
            std::fs::canonicalize(href_as_path)
                .map(Href::Path)
                .map_err(Error::from)
        }
    }

    /// Returns a reference to this href as a (Path)[std::path::Path].
    ///
    /// Returns `None` if it is an (Href::Url).
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("data/catalog.json", None).unwrap();
    /// assert!(href.as_path().is_some());
    /// let href = Href::new("./catalog.json", "http://example.com/stac").unwrap();
    /// assert!(href.as_path().is_none());
    /// ```
    pub fn as_path(&self) -> Option<&Path> {
        match self {
            Href::Path(path) => Some(path.as_path()),
            Href::Url(_) => None,
        }
    }

    /// Returns a reference to this href as a `/`-separated &str.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("data/catalog.json", None).unwrap();
    /// println!("{}", href.to_string());
    /// ```
    pub fn to_string(&self) -> String {
        match self {
            Href::Path(path) => path.to_slash_lossy(),
            Href::Url(url) => url.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Href;

    #[test]
    fn catalog_base() {
        let href = Href::new(
            "./extensions-collection/collection.json",
            "data/catalog.json",
        )
        .unwrap();
        assert!(href
            .to_string()
            .ends_with("/data/extensions-collection/collection.json"))
    }
}
