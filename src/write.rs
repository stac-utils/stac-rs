use crate::{Error, Href, HrefObject, Result};
use path_slash::PathBufExt;
use serde_json::Value;
use std::{
    fs::File,
    io::BufWriter,
    path::{Path, PathBuf},
};
use url::Url;

/// A trait to describe things that can write STAC objects.
pub trait Write {
    /// Writes a [HrefObject], consuming it.
    ///
    /// # Examples
    ///
    /// [Writer] implements [Write]:
    ///
    /// ```no_run
    /// use stac::{Writer, Item, Write, HrefObject};
    /// let object = HrefObject::new(Item::new("an-id"), "item.json");
    /// let writer = Writer::default();
    /// writer.write(object).unwrap();
    /// ```
    fn write(&self, object: HrefObject) -> Result<()> {
        let value = object.object.into_value()?;
        self.write_json(value, &object.href)
    }

    /// Writes a [serde_json::Value] to an href.
    ///
    /// # Examples
    ///
    /// [Writer] implements [Write]:
    ///
    /// ```no_run
    /// use stac::{Writer, Write, Href};
    /// use serde_json::json;
    /// let data = json!({"foo": "bar"});
    /// let writer = Writer::default();
    /// writer.write_json(data, &Href::new("baz.json")).unwrap();
    /// ```
    fn write_json(&self, value: Value, href: &Href) -> Result<()> {
        match href {
            Href::Url(url) => self.write_json_to_url(value, url),
            Href::Path(path) => self.write_json_to_path(value, PathBuf::from_slash(path)),
        }
    }

    /// Writes JSON data to a url.
    ///
    /// # Examples
    ///
    /// [Writer] implements `Write`, but can't write to urls:
    ///
    /// ```
    /// use url::Url;
    /// use stac::{Writer, Write};
    /// use serde_json::json;
    /// let value = json!({"a-key": "a-value"});
    /// let writer = Writer::new();
    /// writer.write_json_to_url(value, &Url::parse("http://stac.test/value.json").unwrap()).unwrap_err();
    /// ```
    fn write_json_to_url(&self, value: Value, url: &Url) -> Result<()>;

    /// Writes JSON data to a path.
    ///
    /// # Examples
    ///
    /// [Writer] implements `Write`:
    ///
    /// ```no_run
    /// use stac::{Writer, Write};
    /// use serde_json::json;
    /// let value = json!({"a-key": "a-value"});
    /// let writer = Writer::new();
    /// writer.write_json_to_path(value, "out.json").unwrap();
    /// ```
    fn write_json_to_path(&self, value: Value, path: impl AsRef<Path>) -> Result<()>;
}

/// The default writer that comes with **stac-rs**.
#[derive(Debug)]
pub struct Writer {
    /// Pretty-print json?
    pub pretty: bool,
}

impl Writer {
    /// Creates a new, default writer.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac::Writer;
    /// let writer = Writer::new();
    /// ```
    pub fn new() -> Writer {
        Writer::default()
    }
}

impl Write for Writer {
    fn write_json_to_url(&self, _: Value, url: &Url) -> Result<()> {
        Err(Error::CannotWriteUrl(url.clone()))
    }

    fn write_json_to_path(&self, value: Value, path: impl AsRef<Path>) -> Result<()> {
        if let Some(parent) = path.as_ref().parent() {
            std::fs::create_dir_all(parent)?;
        }
        let file = File::create(path)?;
        let writer = BufWriter::new(file);
        if self.pretty {
            serde_json::to_writer_pretty(writer, &value).map_err(Error::from)
        } else {
            serde_json::to_writer(writer, &value).map_err(Error::from)
        }
    }
}

impl Default for Writer {
    fn default() -> Writer {
        Writer { pretty: true }
    }
}

#[cfg(test)]
mod tests {
    use super::{Write, Writer};
    use crate::{HrefObject, Item};

    #[test]
    fn write() {
        let item = Item::new("an-item");
        let directory = tempfile::tempdir().unwrap();
        let href = directory.path().join("item.json");
        let object = HrefObject::new(item, href.clone());

        let writer = Writer::default();
        writer.write(object.clone()).unwrap();

        let read_object = crate::read(href).unwrap();
        assert_eq!(read_object, object);
    }
}
