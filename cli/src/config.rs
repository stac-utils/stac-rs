use std::{convert::Infallible, str::FromStr};

/// A collection of configuration entries.
#[derive(Clone, Debug, Default)]
pub(crate) struct Config(Vec<Entry>);

/// `key=value`` pairs for object store configuration
#[derive(Clone, Debug)]
pub(crate) struct Entry {
    key: String,
    value: String,
}

impl Config {
    /// Returns an iterator over this config's key value pairs.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0
            .iter()
            .map(|entry| (entry.key.as_str(), entry.value.as_str()))
    }
}

impl FromStr for Entry {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Infallible> {
        if let Some((key, value)) = s.split_once('=') {
            Ok(Entry {
                key: key.to_string(),
                value: value.to_string(),
            })
        } else {
            Ok(Entry {
                key: s.to_string(),
                value: String::new(),
            })
        }
    }
}

impl From<Vec<Entry>> for Config {
    fn from(value: Vec<Entry>) -> Self {
        Config(value)
    }
}
