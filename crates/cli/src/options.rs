use std::{convert::Infallible, str::FromStr};

/// A collection of configuration entries.
#[derive(Clone, Debug, Default)]
pub(crate) struct Options(Vec<KeyValue>);

/// `key=value`` pairs for object store configuration
#[derive(Clone, Debug)]
pub(crate) struct KeyValue {
    key: String,
    value: String,
}

impl Options {
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        self.0.iter().map(|kv| (kv.key.as_str(), kv.value.as_str()))
    }
}

impl FromStr for KeyValue {
    type Err = Infallible;
    fn from_str(s: &str) -> Result<Self, Infallible> {
        if let Some((key, value)) = s.split_once('=') {
            Ok(KeyValue {
                key: key.to_string(),
                value: value.to_string(),
            })
        } else {
            Ok(KeyValue {
                key: s.to_string(),
                value: String::new(),
            })
        }
    }
}

impl From<Vec<KeyValue>> for Options {
    fn from(value: Vec<KeyValue>) -> Self {
        Options(value)
    }
}

impl From<Options> for Vec<(String, String)> {
    fn from(value: Options) -> Self {
        value.0.into_iter().map(|kv| (kv.key, kv.value)).collect()
    }
}
