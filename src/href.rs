use crate::Error;
use log::warn;
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
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("data", None).unwrap();
    /// assert!(href.to_str().starts_with("/"));
    /// assert_eq!(Href::new("a/path", Some("http://example.com")).unwrap().to_str(), "http://example.com/a/path");
    /// ```
    pub fn new(href: &str, base: Option<&str>) -> Result<Href, Error> {
        if let Ok(url) = Url::parse(href) {
            return Ok(Href::Url(url));
        }
        let href_as_path = Path::new(href);
        if href_as_path.is_absolute() {
            return Ok(Href::Path(href_as_path.to_path_buf()));
        }
        if let Some(base) = base {
            if let Ok(base) = Url::parse(base) {
                base.join(href).map(Href::Url).map_err(Error::from)
            } else {
                let mut base_path_buf = PathBuf::from(base);
                if !base_path_buf.pop() {
                    warn!("No parent in base url: {}", base);
                }
                let href = base_path_buf.join(href);
                std::fs::canonicalize(href)
                    .map(Href::Path)
                    .map_err(Error::from)
            }
        } else {
            std::fs::canonicalize(href)
                .map(Href::Path)
                .map_err(Error::from)
        }
    }

    /// Returns a reference to this href as a &str.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("data/catalog.json", None).unwrap();
    /// println!("{}", href.to_str());
    /// ```
    pub fn to_str(&self) -> &str {
        match self {
            Href::Path(path) => path.to_str().unwrap_or(""),
            Href::Url(url) => url.as_str(),
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
            Some("data/catalog.json"),
        )
        .unwrap();
        assert!(href
            .to_str()
            .ends_with("/data/extensions-collection/collection.json"))
    }
}
