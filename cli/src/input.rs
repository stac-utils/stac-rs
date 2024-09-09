use crate::{Error, Result};
use bytes::Bytes;
use stac::{Format, Href, Value};
use std::{
    fs::File,
    io::{BufReader, Read},
};

/// The input to a CLI run.
#[derive(Clone, Debug)]
pub struct Input {
    /// The format of the input data.
    pub format: Format,

    /// The input file.
    ///
    /// If not provided, this will be read from stdin.
    pub infile: Option<String>,
}

impl Input {
    /// Creates a new input from an optional infile and an optional format.
    ///
    /// # Examples
    ///
    /// ```
    /// use stac_cli::Input;
    ///
    /// let input = Input::new(None, None); // defaults to stdin, json
    /// ```
    pub fn new(infile: Option<String>, format: Option<Format>) -> Result<Input> {
        let infile = infile.and_then(|infile| if infile == "-" { None } else { Some(infile) });
        let format = format
            .or_else(|| infile.as_deref().and_then(Format::infer_from_href))
            .unwrap_or_default();
        Ok(Input { format, infile })
    }

    /// Gets a STAC value from the input.
    ///
    /// Uses the infile that this input was created with, if there was one ... otherwise, gets from stdin.
    pub fn get(&self) -> Result<Value> {
        if let Some(infile) = self.infile.as_ref() {
            let mut file = File::open(infile)?;
            match self.format {
                Format::Json => {
                    let mut buf = Vec::new();
                    let _ = file.read_to_end(&mut buf)?;
                    let mut value: Value = serde_json::from_slice(&buf)?;
                    value.set_href(infile);
                    Ok(value)
                }
                Format::NdJson => {
                    let mut value: Value =
                        stac::ndjson::from_buf_reader(BufReader::new(file))?.into();
                    value.set_href(infile);
                    Ok(value)
                }
                #[cfg(feature = "geoparquet")]
                Format::Geoparquet => {
                    let mut buf = Vec::new();
                    let _ = file.read_to_end(&mut buf)?;
                    stac::geoparquet::from_reader(Bytes::from(buf))
                        .map(Value::from)
                        .map_err(Error::from)
                }
            }
        } else {
            match self.format {
                Format::Json => serde_json::from_reader(std::io::stdin()).map_err(Error::from),
                Format::NdJson => stac::ndjson::from_buf_reader(BufReader::new(std::io::stdin()))
                    .map(Value::from)
                    .map_err(Error::from),
                #[cfg(feature = "geoparquet")]
                Format::Geoparquet => {
                    let mut buf = Vec::new();
                    let _ = std::io::stdin().read_to_end(&mut buf)?;
                    stac::geoparquet::from_reader(Bytes::from(buf))
                        .map(Value::from)
                        .map_err(Error::from)
                }
            }
        }
    }
}
