use crate::{Error, Result};
use object_store::{local::LocalFileSystem, path::Path, ObjectStore};
use url::Url;

/// Parse an href into an object store.
///
/// Handles local paths as well.
pub(crate) fn parse_href_opts<I, K, V>(
    href: &str,
    options: I,
) -> Result<(Box<dyn ObjectStore>, Path)>
where
    I: IntoIterator<Item = (K, V)>,
    K: AsRef<str>,
    V: Into<String>,
{
    if let Some(url) = Url::parse(href).ok() {
        object_store::parse_url_opts(&url, options).map_err(Error::from)
    } else {
        let path = Path::from_filesystem_path(href)?;
        let object_store = LocalFileSystem::new();
        Ok((Box::new(object_store), path))
    }
}
