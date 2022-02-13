use crate::{Error, Object, PathBufHref};
use serde_json::Value;
use std::{fs::File, io::BufWriter};

/// A trait to describe things that can write STAC objects.
pub trait Write {
    /// Writes an [Object], consuming it.
    ///
    /// # Examples
    ///
    /// [Writer] implements [Write]:
    ///
    /// ```no_run
    /// use stac::{Writer, Item, Write, Object};
    /// let object = Object::new(Item::new("an-id"), "item.json").unwrap();
    /// let writer = Writer::default();
    /// writer.write(object).unwrap();
    /// ```
    fn write(&self, mut object: Object) -> Result<(), Error> {
        if let Some(href) = object.href.take() {
            let value = object.into_value()?;
            self.write_value(value, href)
        } else {
            Err(Error::MissingHref)
        }
    }

    /// Writes a [serde_json::Value] to an href.
    ///
    /// # Examples
    ///
    /// [Writer] implements [Write]:
    ///
    /// ```no_run
    /// use stac::{Writer, Write};
    /// use serde_json::json;
    /// let data = json!({"foo": "bar"});
    /// let writer = Writer::default();
    /// writer.write_value(data, "baz.json").unwrap();
    /// ```
    fn write_value<T>(&self, value: Value, href: T) -> Result<(), Error>
    where
        T: Into<PathBufHref>;
}

/// The default writer that comes with **stac-rs**.
#[derive(Debug)]
pub struct Writer {
    /// Pretty-print json?
    pub pretty: bool,
}

impl Write for Writer {
    fn write_value<T>(&self, value: Value, href: T) -> Result<(), Error>
    where
        T: Into<PathBufHref>,
    {
        match href.into() {
            PathBufHref::Path(path) => {
                if let Some(parent) = path.parent() {
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
            PathBufHref::Url(url) => Err(Error::CannotWriteUrl(url)),
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
    use crate::{Item, Object};

    #[test]
    fn write() {
        let item = Item::new("an-item");
        let directory = tempfile::tempdir().unwrap();
        let href = directory.path().join("item.json");
        let object = Object::new(item, href.clone()).unwrap();

        let writer = Writer::default();
        writer.write(object.clone()).unwrap();

        let read_object = crate::read(href).unwrap();
        assert_eq!(read_object, object);
    }

    #[test]
    fn write_no_href() {
        let item = Item::new("an-item");
        let object = Object {
            href: None,
            inner: item.into(),
        };

        let writer = Writer::default();
        let _ = writer.write(object).unwrap_err();
    }
}
