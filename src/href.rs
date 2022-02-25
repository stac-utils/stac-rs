use crate::Error;
use path_slash::PathBufExt;
use std::path::PathBuf;
use url::Url;

/// An href can be an absolute url, an absolute path, or a relative path.
///
/// Hrefs are used throughout the STAC specification to link between objects, to
/// assets, and to outside resources. They are defined in the specification as
/// URIs, meaning they should always be `/`-delimited paths. This `Href` enum
/// provides a platform-independent way to store and manipulate the paths.
///
/// `Href`s are always created from `/`-delimited strings. If you might be
/// working with a `\` delimited string (e.g. on Windows), use [PathBufHref].
///
/// ```
/// use stac::Href;
/// let path_href = Href::new("a/path/to/an/item.json");
/// let url_href = Href::new("http://example.com/item.json");
///
/// #[cfg(target_os = "windows")]
/// {
///     use stac::PathBufHref;
///     let path_buf_href = PathBufHref::new(r"a\path\to\an\item.json");
///     let href = Href::from(path_buf_href);
///     assert_eq!(href.as_str(), "a/path/to/an/item.json");
/// }
/// ```
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Href {
    /// A parsed url href.
    Url(Url),

    /// A path href, either relative or absolute.
    ///
    /// This path will be `/`-delimited regardless of OS.
    Path(String),
}

/// An href that uses [PathBuf] instead of a [String] for paths.
///
/// `PathBufHref` is used when actually reading or writing from hrefs, e.g. in
/// the signature of [Read::read](crate::Read::read). `PathBufHref` can be
/// converted from and to [Hrefs](Href), which uses
/// [path-slash](https://github.com/rhysd/path-slash) to convert the `/`
/// delimiters.
///
/// ```
/// use stac::{Href, PathBufHref};
/// let href = Href::new("data/catalog.json");
/// let path_buf_href = PathBufHref::from(href);
/// let href = Href::from(path_buf_href);
/// ```
#[derive(Debug, Clone)]
pub enum PathBufHref {
    /// A parsed url href.
    Url(Url),

    /// A filesystem path, stored as a [PathBuf].
    Path(PathBuf),
}

impl Href {
    /// Creates an href.
    ///
    /// Does not do any `\` conversion. If you need to handle possibly-`\`-delimited paths, use [PathBufHref].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("data");
    /// assert!(href.is_path());
    /// assert_eq!(href.as_str(), "data");
    /// let href = Href::new("http://example.com/data");
    /// assert!(href.is_url());
    /// assert_eq!(href.as_str(), "http://example.com/data");
    /// ```
    pub fn new<S: ToString>(href: S) -> Href {
        let href = href.to_string();
        if let Ok(url) = Url::parse(&href) {
            if url.cannot_be_a_base() {
                Href::Path(href)
            } else {
                Href::Url(url)
            }
        } else {
            Href::Path(href)
        }
    }

    /// Joins this href to another href.
    ///
    /// If the passed href is an absolute path or a url, this method just
    /// returns the passed href as is.  Otherwise, it builds a new path or url
    /// by joining this href and the provided href.  If `self` ends in a `/` it
    /// is unmodified, otherwise the last segment is treated as a file name and
    /// dropped.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Href;
    /// let base = Href::new("data/catalog.json");
    /// let item = base.join("./item/item.json").unwrap();
    /// assert_eq!(item.as_str(), "data/item/item.json");
    /// let absolute_item = base.join("http://example.com/data/item.json").unwrap();
    /// assert_eq!(absolute_item.as_str(), "http://example.com/data/item.json");
    ///
    /// let base = Href::new("data/");
    /// let item = base.join("./item/item.json").unwrap();
    /// assert_eq!(item.as_str(), "data/item/item.json");
    /// ```
    pub fn join<T>(&self, href: T) -> Result<Href, Error>
    where
        T: Into<Href>,
    {
        let href = href.into();
        if href.is_absolute() {
            return Ok(href);
        }
        match self {
            Href::Url(base) => base.join(href.as_str()).map(Href::Url).map_err(Error::from),
            Href::Path(base) => {
                let (path, _) = extract_path_filename(base);
                let path = if path.is_empty() {
                    href.into_string()
                } else {
                    format!("{}/{}", path, href.as_str())
                };
                let path = normalize_path(path);
                Ok(Href::Path(path))
            }
        }
    }

    /// Returns `true` if this href is a url.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("http://example.com");
    /// assert!(href.is_url());
    /// let href = Href::new("data/catalog.json");
    /// assert!(!href.is_url());
    /// ```
    pub fn is_url(&self) -> bool {
        matches!(self, Href::Url(_))
    }

    /// Returns a reference to this href as a [Url](url::Url), if it is one.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Href;
    /// use url::Url;
    /// let href = Href::new("http://example.com/catalog.json");
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

    /// Returns `true` if this href is a path.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let href = Href::new("http://example.com");
    /// assert!(!href.is_path());
    /// let href = Href::new("data/catalog.json");
    /// assert!(href.is_path());
    /// ```
    pub fn is_path(&self) -> bool {
        matches!(self, Href::Path(_))
    }

    /// Returns this href as a [&str].
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Href;
    /// let mut href = Href::new("data/catalog.json");
    /// assert_eq!(href.as_str(), "data/catalog.json");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Href::Url(url) => url.as_str(),
            Href::Path(path) => path,
        }
    }

    /// Returns `true` if this is an absolute href.
    ///
    /// [Href::Urls](Href::Url) are always absolute.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Href;
    /// assert!(Href::new("http://example.com/data").is_absolute());
    /// assert!(Href::new("/an/absolute/path").is_absolute());
    /// assert!(!Href::new("a/relative/path").is_absolute());
    /// ```
    pub fn is_absolute(&self) -> bool {
        match self {
            Href::Url(_) => true,
            Href::Path(path) => is_absolute(path),
        }
    }

    /// Converts this href into an absolute one.
    ///
    /// For [Href::Url], this is a noop.  For [Href::Path], this will return an
    /// error if [std::fs::canonicalize] fails.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Href;
    /// let mut href = Href::new("http://example.com/data");
    /// href.make_absolute().unwrap();
    /// assert_eq!(href.as_str(), "http://example.com/data");
    ///
    /// let mut href = Href::new("data/catalog.json");
    /// href.make_absolute().unwrap();
    /// let err = Href::new("not/a/real/path").make_absolute().unwrap_err();
    ///
    pub fn make_absolute(&mut self) -> Result<(), Error> {
        if let Href::Path(path) = self {
            if let PathBufHref::Path(path) = PathBufHref::from(path.as_str()) {
                let path = std::fs::canonicalize(path)?;
                *self = PathBufHref::Path(path).into();
            }
        }
        Ok(())
    }

    /// Converts the provided href into a relative href, relative to `self`.
    ///
    /// - Adds an `"./"` to the front of "downward" hrefs.
    /// - If both paths don't share a common base, they are assumed to be in the
    /// same parent directory.
    /// - If both paths are absolute and they do not share a common base,
    /// returns `None`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Href;
    /// let catalog = Href::new("data/catalog.json");
    /// let collection = Href::new("data/collection/collection.json");
    /// assert_eq!(
    ///     catalog.make_relative(collection.clone()).unwrap().as_str(),
    ///     "./collection/collection.json"
    /// );
    /// assert_eq!(
    ///     collection.make_relative(catalog).unwrap().as_str(),
    ///     "../catalog.json"
    /// );
    pub fn make_relative(&self, href: Href) -> Option<Href> {
        match self {
            Href::Url(base) => match href {
                Href::Url(url) => base.make_relative(&url).map(|path| {
                    if path.is_empty() {
                        let (_, file_name) = extract_path_filename(url.path());
                        Href::Path(format!("./{}", file_name))
                    } else {
                        Href::Path(path)
                    }
                }),
                // We skip the leading slash on the path to get make relative to go.
                Href::Path(path) => {
                    if is_absolute(&path) {
                        None
                    } else {
                        Some(Href::Path(make_relative(&base.path()[1..], &path)))
                    }
                }
            },
            Href::Path(base) => match href {
                Href::Url(url) => Some(Href::Url(url)),
                Href::Path(path) => {
                    if is_absolute(base) && is_absolute(&path) {
                        None
                    } else {
                        Some(Href::Path(make_relative(base, &path)))
                    }
                }
            },
        }
    }

    /// Rebases a relative href from one root to another.
    ///
    /// If `self` is a url or absolute, this is a noop.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Href;
    /// let old_root_catalog = Href::new("path/to/a/catalog.json");
    /// let new_root = Href::new("a/new/base/");
    /// let mut item = Href::new("path/to/a/item/item.json");
    /// item.rebase(&old_root_catalog, &new_root).unwrap();
    /// assert_eq!(item.as_str(), "a/new/base/item/item.json");
    /// ```
    pub fn rebase(&mut self, from: &Href, to: &Href) -> Result<(), Error> {
        if let Href::Path(path) = self {
            if is_absolute(path) {
                return Ok(());
            }
            *self = to.join(make_relative(from.as_str(), path))?;
        }
        Ok(())
    }

    /// Returns this href's file name.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Href;
    /// let href = Href::new("a/path/item.json");
    /// assert_eq!(href.file_name(), "item.json");
    /// ```
    pub fn file_name(&self) -> &str {
        extract_path_filename(self.as_str()).1
    }

    fn into_string(self) -> String {
        match self {
            Href::Path(path) => path,
            Href::Url(url) => url.into(),
        }
    }
}

impl PathBufHref {
    fn new<T: ToString>(href: T) -> PathBufHref {
        Href::new(href).into()
    }
}

impl From<Url> for Href {
    fn from(url: Url) -> Href {
        Href::Url(url)
    }
}

impl From<Href> for PathBufHref {
    fn from(href: Href) -> PathBufHref {
        match href {
            Href::Url(url) => PathBufHref::Url(url),
            Href::Path(path) => PathBufHref::Path(PathBuf::from_slash(path)),
        }
    }
}

impl From<PathBufHref> for Href {
    fn from(href: PathBufHref) -> Href {
        match href {
            PathBufHref::Url(url) => Href::Url(url),
            PathBufHref::Path(path) => Href::Path(path.to_slash_lossy()),
        }
    }
}

impl From<PathBuf> for PathBufHref {
    fn from(path_buf: PathBuf) -> PathBufHref {
        PathBufHref::Path(path_buf)
    }
}

impl From<PathBuf> for Href {
    fn from(path_buf: PathBuf) -> Href {
        PathBufHref::Path(path_buf).into()
    }
}

impl From<&str> for Href {
    fn from(s: &str) -> Href {
        Href::new(s)
    }
}

impl From<&String> for Href {
    fn from(s: &String) -> Href {
        Href::new(s)
    }
}

impl From<String> for Href {
    fn from(s: String) -> Href {
        Href::new(s)
    }
}

impl From<&str> for PathBufHref {
    fn from(s: &str) -> PathBufHref {
        PathBufHref::new(s)
    }
}

impl From<&Href> for PathBufHref {
    fn from(href: &Href) -> PathBufHref {
        PathBufHref::new(href.as_str())
    }
}

impl From<&String> for PathBufHref {
    fn from(s: &String) -> PathBufHref {
        PathBufHref::new(s)
    }
}

impl From<String> for PathBufHref {
    fn from(s: String) -> PathBufHref {
        PathBufHref::new(s)
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

fn extract_path_filename(s: &str) -> (&str, &str) {
    let last_slash_idx = s.rfind('/').unwrap_or(0);
    let (path, filename) = s.split_at(last_slash_idx);
    if filename.is_empty() {
        (path, "")
    } else {
        (path, &filename[1..])
    }
}

fn make_relative(base: &str, target: &str) -> String {
    // Taken from https://docs.rs/url/latest/src/url/lib.rs.html#401-490
    let mut relative = String::new();

    let (base_path, _) = extract_path_filename(base);
    let (target_path, target_filename) = extract_path_filename(target);

    let mut base_path = base_path.split('/').peekable();
    let mut target_path = target_path.split('/').peekable();

    while base_path.peek().is_some() && base_path.peek() == target_path.peek() {
        let _ = base_path.next();
        let _ = target_path.next();
    }

    for base_path_segment in base_path {
        if base_path_segment.is_empty() {
            break;
        }

        if !relative.is_empty() {
            relative.push('/');
        }

        relative.push_str("..");
    }

    for target_path_segment in target_path {
        if !relative.is_empty() {
            relative.push('/');
        }

        relative.push_str(target_path_segment);
    }

    if target_filename.is_empty() {
        relative.push('/');
    } else {
        if !relative.is_empty() {
            relative.push('/');
        }
        relative.push_str(target_filename);
    }

    if !relative.starts_with("..") {
        relative.insert_str(0, "./");
    }
    relative
}

#[cfg(test)]
mod tests {
    use super::Href;
    use url::Url;

    #[test]
    fn new_path() {
        let href = Href::new("data/catalog.json");
        assert!(href.is_path());
        assert_eq!(href.as_str(), "data/catalog.json");
    }

    #[test]
    fn new_url() {
        let href = Href::new("http://example.com/catalog.json");
        assert_eq!(
            href.as_url().unwrap(),
            &Url::parse("http://example.com/catalog.json").unwrap()
        );
    }

    #[test]
    fn join_path() {
        let href = Href::new("data/catalog.json");
        assert!(href.is_path());
        assert_eq!(
            href.join("./extensions-collection/collection.json")
                .unwrap()
                .as_str(),
            "data/extensions-collection/collection.json",
        );
        let href = Href::new("data/");
        assert_eq!(
            href.join("catalog.json").unwrap().as_str(),
            "data/catalog.json"
        );
    }

    #[test]
    fn join_empty_path() {
        let href = Href::new("");
        assert!(href.is_path());
        assert_eq!(href.join("catalog.json").unwrap().as_str(), "catalog.json",);
    }

    #[test]
    fn join_absolute_path() {
        let href = Href::new("data/catalog.json");
        assert!(href.is_path());
        assert_eq!(
            href.join("/an/absolute/path/item.json").unwrap().as_str(),
            "/an/absolute/path/item.json"
        );
    }

    #[test]
    fn join_url() {
        let href = Href::new("http://example.com/data/catalog.json");
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
        let href = Href::new("data/catalog.json");
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
        let href = Href::new("data/catalog.json");
        assert_eq!(href.as_str(), "data/catalog.json");
    }

    #[test]
    fn url_to_string() {
        let href = Href::new("http://example.com/catalog.json");
        assert_eq!(href.as_str(), "http://example.com/catalog.json");
    }

    #[test]
    fn make_path_relative() {
        let base = Href::new("data/catalog.json");
        let target = Href::new("data/extensions-collection/collection.json");
        assert_eq!(
            base.make_relative(target.clone()).unwrap().as_str(),
            "./extensions-collection/collection.json"
        );
        assert_eq!(
            target.make_relative(base.clone()).unwrap().as_str(),
            "../catalog.json"
        );
        assert_eq!(
            base.make_relative(base.clone()).unwrap().as_str(),
            "./catalog.json"
        );
        assert_eq!(
            base.make_relative(Href::new("http://example.com/item.json"))
                .unwrap()
                .as_str(),
            "http://example.com/item.json"
        );
    }

    #[test]
    fn make_path_relative_no_common_base() {
        let base = Href::new("data/catalog.json");
        let target = Href::new("other/extensions-collection/collection.json");
        assert_eq!(
            base.make_relative(target.clone()).unwrap().as_str(),
            "../other/extensions-collection/collection.json"
        );

        let base = Href::new("/data/catalog.json");
        let target = Href::new("/other/extensions-collection/collection.json");
        assert!(base.make_relative(target.clone()).is_none());

        let base = Href::new("http://example.com/catalog.json");
        let target = Href::new("http://example.org/item/item.json");
        assert!(base.make_relative(target.clone()).is_none());
    }

    #[test]
    fn make_url_relative() {
        let base = Href::new("http://example.com/data/catalog.json");
        let target = Href::new("data/extensions-collection/collection.json");
        assert_eq!(
            base.make_relative(target.clone()).unwrap().as_str(),
            "./extensions-collection/collection.json"
        );
        assert_eq!(
            base.make_relative(base.clone()).unwrap().as_str(),
            "./catalog.json"
        );

        let target = Href::new("/data/extensions-collection/collection.json");
        assert!(base.make_relative(target).is_none());
    }

    #[test]
    fn rebase_path_relative() {
        let old_root_catalog = Href::new("path/to/a/catalog.json");
        let new_root = Href::new("a/new/base/");
        let mut item = Href::new("path/to/a/item/item.json");
        item.rebase(&old_root_catalog, &new_root).unwrap();
        assert_eq!(item.as_str(), "a/new/base/item/item.json");
    }
}
