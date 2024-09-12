use crate::{options::Options, Error, Result};
use stac::{io::Config, Format, Value};

/// The input to a CLI run.
#[derive(Debug, Default)]
pub(crate) struct Input {
    config: Config,
    href: Option<String>,
}

impl Input {
    /// Creates a new input.
    pub(crate) fn new(
        href: impl Into<Option<String>>,
        format: impl Into<Option<Format>>,
        options: impl Into<Options>,
    ) -> Input {
        let href = href
            .into()
            .and_then(|href| if href == "-" { None } else { Some(href) });
        let config = Config::new().format(format).options(options.into());
        Input { config, href }
    }

    /// Creates a new input with the given href.
    pub(crate) fn with_href(&self, href: impl Into<Option<String>>) -> Input {
        Input {
            config: self.config.clone(),
            href: href.into(),
        }
    }

    /// Gets a STAC value from the input.
    pub(crate) async fn get(&self) -> Result<Value> {
        if let Some(href) = self.href.as_ref() {
            self.config.get(href.clone()).await.map_err(Error::from)
        } else {
            self.config
                .from_reader(std::io::stdin())
                .map_err(Error::from)
        }
    }
}
