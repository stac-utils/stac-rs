use crate::{Error, Result};
use path_slash::{PathBufExt, PathExt};
use std::{
    convert::Infallible,
    fmt::{Display, Formatter},
    path::{Path, PathBuf},
    str::FromStr,
};
use url::Url;

/// An href can be an absolute url, an absolute path, or a relative path.
///
/// Hrefs are used throughout the STAC specification to link between objects, to
/// assets, and to outside resources. They are defined in the specification as
/// URIs, meaning they should always be `/`-delimited paths. This `Href` enum
/// provides a platform-independent way to store and manipulate the paths.
///
/// `Href`s are always created from `/`-delimited strings. If you might be
/// working with a `\` delimited string (e.g. on Windows), use [Href::to_slash].
///
/// ```
/// use stac::Href;
/// use std::path::PathBuf;
/// let path_href = Href::new("a/path/to/an/item.json");
/// let url_href = Href::new("http://example.com/item.json");
///
/// #[cfg(target_os = "windows")]
/// {
///     let href = Href::to_slash(r"a\path\to\an\item.json");
///     assert_eq!(href.as_str(), "a/path/to/an/item.json");
/// }
/// ```
#[derive(Debug, PartialEq, Clone, Eq, Hash)]
pub enum Href {
    /// A parsed url href.
    Url(Url),

    /// A path href, either relative or absolute.
    ///
    /// This path is always `/`-delimited.
    Path(String),
}

impl Href {
    /// Creates an href.
    ///
    /// Does not do any `\` conversion. If you need to handle
    /// possibly-`\`-delimited paths, use [Href::to_slash].
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
    pub fn new(href: impl ToString) -> Href {
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

    /// Creates an href from a possibly `\`-delimited string.
    ///
    /// If the href is an url, no conversion is performed. Otherwise, the path
    /// is converted from `\`-delimited to `/`-delimited.
    pub fn to_slash(href: impl ToString) -> Href {
        let href = Href::new(href);
        if let Href::Path(path) = href {
            Path::new(&path).into()
        } else {
            href
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
    pub fn join(&self, href: impl Into<Href>) -> Result<Href> {
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
    pub fn make_absolute(&mut self) -> Result<()> {
        if let Href::Path(path) = self {
            let path = PathBuf::from_slash(path).canonicalize()?;
            *self = Href::Path(path.to_slash_lossy());
        }
        Ok(())
    }

    /// Converts the provided href into a relative href, relative to `self`.
    ///
    /// - Adds an `"./"` to the front of "downward" hrefs.
    /// - If both paths don't share a common base, they are assumed to be in the
    /// same parent directory.
    /// - If both paths are absolute and they do not share a common base,
    /// returns the original href unchanged.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Href;
    /// let catalog = Href::new("data/catalog.json");
    /// let collection = Href::new("data/collection/collection.json");
    /// assert_eq!(
    ///     catalog.make_relative(collection.clone()).as_str(),
    ///     "./collection/collection.json"
    /// );
    /// assert_eq!(
    ///     collection.make_relative(catalog).as_str(),
    ///     "../catalog.json"
    /// );
    pub fn make_relative(&self, href: Href) -> Href {
        match self {
            Href::Url(base) => match href {
                Href::Url(url) => base
                    .make_relative(&url)
                    .map(|path| {
                        if path.is_empty() {
                            let (_, file_name) = extract_path_filename(url.path());
                            Href::Path(format!("./{}", file_name))
                        } else {
                            Href::Path(path)
                        }
                    })
                    .unwrap_or_else(|| Href::Url(url)),
                // We skip the leading slash on the path to get make relative to go.
                Href::Path(path) => {
                    if is_absolute(&path) {
                        Href::Path(path)
                    } else {
                        Href::Path(make_relative(&base.path()[1..], &path))
                    }
                }
            },
            Href::Path(base) => match href {
                Href::Url(url) => Href::Url(url),
                Href::Path(path) => {
                    if is_absolute(base) && is_absolute(&path) {
                        Href::Path(path)
                    } else {
                        Href::Path(make_relative(base, &path))
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
    pub fn rebase(&mut self, from: &Href, to: &Href) -> Result<()> {
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

    /// Returns this href's directory.
    ///
    /// # Examples
    ///
    /// ```
    /// # use stac::Href;
    /// let href = Href::new("a/path/item.json");
    /// assert_eq!(href.directory(), "a/path");
    /// ```
    pub fn directory(&self) -> &str {
        extract_path_filename(self.as_str()).0
    }

    pub(crate) fn ensure_ends_in_slash(&mut self) {
        match self {
            Href::Url(url) => {
                if let Ok(mut path_segments) = url.path_segments_mut() {
                    let _ = path_segments.push("/");
                }
            }
            Href::Path(path) => path.push('/'),
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

impl From<&Path> for Href {
    fn from(path: &Path) -> Href {
        Href::Path(path.to_slash_lossy())
    }
}

impl From<PathBuf> for Href {
    fn from(path: PathBuf) -> Href {
        Href::Path(path.to_slash_lossy())
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

impl From<Href> for String {
    fn from(href: Href) -> String {
        match href {
            Href::Url(url) => url.to_string(),
            Href::Path(path) => path,
        }
    }
}

impl FromStr for Href {
    type Err = Infallible;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        Ok(Href::new(s))
    }
}

impl Display for Href {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
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
            base.make_relative(target.clone()).as_str(),
            "./extensions-collection/collection.json"
        );
        assert_eq!(
            target.make_relative(base.clone()).as_str(),
            "../catalog.json"
        );
        assert_eq!(base.make_relative(base.clone()).as_str(), "./catalog.json");
        assert_eq!(
            base.make_relative(Href::new("http://example.com/item.json"))
                .as_str(),
            "http://example.com/item.json"
        );
    }

    #[test]
    fn make_path_relative_no_common_base() {
        let base = Href::new("data/catalog.json");
        let target = Href::new("other/extensions-collection/collection.json");
        assert_eq!(
            base.make_relative(target.clone()).as_str(),
            "../other/extensions-collection/collection.json"
        );

        let base = Href::new("/data/catalog.json");
        let target = Href::new("/other/extensions-collection/collection.json");
        assert_eq!(base.make_relative(target.clone()), target);

        let base = Href::new("http://example.com/catalog.json");
        let target = Href::new("http://example.org/item/item.json");
        assert_eq!(base.make_relative(target.clone()), target);
    }

    #[test]
    fn make_url_relative() {
        let base = Href::new("http://example.com/data/catalog.json");
        let target = Href::new("data/extensions-collection/collection.json");
        assert_eq!(
            base.make_relative(target.clone()).as_str(),
            "./extensions-collection/collection.json"
        );
        assert_eq!(base.make_relative(base.clone()).as_str(), "./catalog.json");

        let target = Href::new("/data/extensions-collection/collection.json");
        assert_eq!(base.make_relative(target.clone()), target);
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
