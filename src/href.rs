use crate::Error;
use path_slash::{PathBufExt, PathExt};
use std::path::{Path, PathBuf};
use url::{ParseError, Url};

/// A wrapper around a parsed href.
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Href {
    /// A parsed url href.
    Url(Url),

    /// A filesystem path.
    ///
    /// This path will be '/'-delimited regardless of OS.
    Path(String),
}

impl Href {
    /// Creates an href from a '/'-delimited string.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("data").unwrap();
    /// assert_eq!(href.to_path().unwrap(), std::path::Path::new("data"));
    /// ```
    pub fn new(href: &str) -> Result<Href, Error> {
        match Url::parse(href) {
            Ok(url) => Ok(Href::Url(url)),
            Err(err) => match err {
                ParseError::RelativeUrlWithoutBase => {
                    Ok(Href::Path(Path::new(href).to_slash_lossy()))
                }
                _ => Err(Error::from(err)),
            },
        }
    }

    /// Joins this href to another href.
    ///
    /// If the provided href is an absolute path or a url, just return that.
    /// Otherwise, build a new path/url with the provided href.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    ///
    /// let base = Href::new("data/catalog.json").unwrap();
    /// let item = base.join("./extensions-collection/collection.json").unwrap();
    /// assert_eq!(
    ///     item.to_path().unwrap(),
    ///     std::path::Path::new("data/extensions-collection/collection.json")
    /// );
    /// ```
    pub fn join<T, E>(&self, href: T) -> Result<Href, Error>
    where
        T: TryInto<Href, Error = E>,
        Error: From<E>,
    {
        let href = href.try_into()?;
        if href.is_url() || href.is_absolute_path() {
            return Ok(href);
        }
        match self {
            Href::Url(base) => {
                let href = base.join(href.as_str()).map(Href::from)?;
                Ok(href)
            }
            Href::Path(base) => {
                // Inspired by/taken from the url crate
                let last_slash_index = base.rfind('/').unwrap_or(0);
                let (directory, _) = base.split_at(last_slash_index);
                let path = if directory.is_empty() {
                    href.into_string()
                } else {
                    format!("{}/{}", directory, href.as_str())
                };
                let path = normalize_path(path);
                Ok(Href::Path(path))
            }
        }
    }

    /// Converts this href to a (PathBuf)[std::path::PathBuf].
    ///
    /// Returns [None] if the href is not on the local filesystem.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("data/catalog.json").unwrap();
    /// assert!(href.to_path().is_some());
    /// let href = Href::new("http://example.com/stac").unwrap();
    /// assert!(href.to_path().is_none());
    /// ```
    pub fn to_path(&self) -> Option<PathBuf> {
        match self {
            Href::Url(_) => None,
            Href::Path(path) => Some(PathBuf::from_slash(path)),
        }
    }

    /// Returns a reference to this href as a (Url)[url::Url].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// use url::Url;
    ///
    /// let href = Href::new("http://example.com/catalog.json").unwrap();
    /// assert_eq!(
    ///     href.as_url().unwrap(),
    ///     &Url::parse("http://example.com/catalog.json").unwrap()
    /// );
    /// ```
    pub fn as_url(&self) -> Option<&Url> {
        match self {
            Href::Url(url) => Some(url),
            Href::Path(_) => None,
        }
    }

    /// Returns true if this href is a url.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("http://example.com").unwrap();
    /// assert!(href.is_url());
    /// let href = Href::new("data/catalog.json").unwrap();
    /// assert!(!href.is_url());
    /// ```
    pub fn is_url(&self) -> bool {
        matches!(self, Href::Url(_))
    }

    /// Returns a reference to this href as a str.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let mut href = Href::new("data/catalog.json").unwrap();
    /// assert_eq!(href.as_str(), "data/catalog.json");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Href::Url(url) => url.as_str(),
            Href::Path(path) => path,
        }
    }

    fn is_absolute_path(&self) -> bool {
        match self {
            Href::Path(path) => is_absolute(path),
            _ => false,
        }
    }

    fn into_string(self) -> String {
        match self {
            Href::Path(path) => path,
            Href::Url(url) => url.into(),
        }
    }
}

impl From<Url> for Href {
    fn from(url: Url) -> Href {
        Href::Url(url)
    }
}

impl TryFrom<&str> for Href {
    type Error = Error;

    fn try_from(s: &str) -> Result<Href, Error> {
        Href::new(s)
    }
}

impl TryFrom<&String> for Href {
    type Error = Error;

    fn try_from(s: &String) -> Result<Href, Error> {
        Href::new(s)
    }
}

fn normalize_path(path: String) -> String {
    let mut parts = if is_absolute(&path) {
        vec![""]
    } else {
        Vec::new()
    };
    for part in path.split('/') {
        match part {
            "." => {}
            ".." => {
                let _ = parts.pop();
            }
            s => parts.push(s),
        }
    }
    parts.join("/")
}

fn is_absolute(path: &str) -> bool {
    path.starts_with('/')
}

#[cfg(test)]
mod tests {
    use super::Href;
    use std::path::Path;
    use url::Url;

    #[test]
    fn new_path() {
        let href = Href::new("data/catalog.json").unwrap();
        assert_eq!(href.to_path().unwrap(), Path::new("data/catalog.json"));
    }

    #[test]
    fn new_url() {
        let href = Href::new("http://example.com/catalog.json").unwrap();
        assert_eq!(
            href.as_url().unwrap(),
            &Url::parse("http://example.com/catalog.json").unwrap()
        );
    }

    #[test]
    fn join_path() {
        let href = Href::new("data/catalog.json").unwrap();
        assert_eq!(
            href.join("./extensions-collection/collection.json")
                .unwrap()
                .to_path()
                .unwrap(),
            Path::new("data/extensions-collection/collection.json"),
        );
    }

    #[test]
    fn join_empty_path() {
        let href = Href::new("").unwrap();
        assert_eq!(
            href.join("catalog.json").unwrap().to_path().unwrap(),
            Path::new("catalog.json"),
        );
    }

    #[test]
    fn join_absolute_path() {
        let href = Href::new("data/catalog.json").unwrap();
        assert_eq!(
            href.join("/an/absolute/path/item.json")
                .unwrap()
                .to_path()
                .unwrap(),
            Path::new("/an/absolute/path/item.json")
        );
    }

    #[test]
    fn join_url() {
        let href = Href::new("http://example.com/data/catalog.json").unwrap();
        assert_eq!(
            href.join("./extensions-collection/collection.json")
                .unwrap()
                .as_url()
                .unwrap(),
            &Url::parse("http://example.com/data/extensions-collection/collection.json").unwrap(),
        );
    }

    #[test]
    fn join_absolute_url() {
        let href = Href::new("data/catalog.json").unwrap();
        assert_eq!(
            href.join("http://example.com/data/catalog.json")
                .unwrap()
                .as_url()
                .unwrap(),
            &Url::parse("http://example.com/data/catalog.json").unwrap()
        );
    }

    #[test]
    fn path_to_string() {
        let href = Href::new("data/catalog.json").unwrap();
        assert_eq!(href.as_str(), "data/catalog.json");
    }

    #[test]
    fn url_to_string() {
        let href = Href::new("http://example.com/catalog.json").unwrap();
        assert_eq!(href.as_str(), "http://example.com/catalog.json");
    }
}
