use crate::{output::Format, Error, Result};
use bytes::Bytes;
use serde::Serialize;

/// An output value, which can either be a [serde_json::Value] or a [stac::Value].
#[derive(Debug, Serialize)]
#[serde(untagged)]
pub enum Value {
    /// A STAC value.
    Stac(stac::Value),

    /// A JSON value.
    Json(serde_json::Value),
}

impl Value {
    pub(crate) fn into_bytes(self, format: Format) -> Result<Bytes> {
        match self {
            Self::Stac(value) => match format {
                Format::CompactJson => serde_json::to_vec(&value)
                    .map(Bytes::from)
                    .map_err(Error::from),
                Format::PrettyJson => serde_json::to_vec_pretty(&value)
                    .map(Bytes::from)
                    .map_err(Error::from),
                Format::NdJson => {
                    if let stac::Value::ItemCollection(item_collection) = value {
                        let mut buf = Vec::new();
                        for item in item_collection.items {
                            serde_json::to_writer(&mut buf, &item)?;
                            buf.push(b'\n');
                        }
                        Ok(buf.into())
                    } else {
                        serde_json::to_vec(&value)
                            .map(Bytes::from)
                            .map_err(Error::from)
                    }
                }
                #[cfg(feature = "geoparquet")]
                Format::Geoparquet(compression) => geoparquet_bytes(value, compression),
            },
            Self::Json(value) => match format {
                Format::CompactJson => serde_json::to_vec(&value)
                    .map(Bytes::from)
                    .map_err(Error::from),
                Format::PrettyJson => serde_json::to_vec_pretty(&value)
                    .map(Bytes::from)
                    .map_err(Error::from),
                Format::NdJson => {
                    if let serde_json::Value::Array(array) = value {
                        let mut buf = Vec::new();
                        for value in array {
                            serde_json::to_writer(&mut buf, &value)?;
                            buf.push(b'\n');
                        }
                        Ok(buf.into())
                    } else {
                        serde_json::to_vec(&value)
                            .map(Bytes::from)
                            .map_err(Error::from)
                    }
                }
                #[cfg(feature = "geoparquet")]
                Format::Geoparquet(compression) => {
                    geoparquet_bytes(serde_json::from_value(value)?, compression)
                }
            },
        }
    }
}

#[cfg(feature = "geoparquet")]
fn geoparquet_bytes(value: stac::Value, compression: parquet::basic::Compression) -> Result<Bytes> {
    tracing::debug!(
        "converting STAC {} to geoparquet bytes using {} compression",
        value.type_name(),
        compression
    );
    let mut options = geoarrow::io::parquet::GeoParquetWriterOptions::default();
    let writer_properties = parquet::file::properties::WriterProperties::builder()
        .set_compression(compression)
        .build();
    options.writer_properties = Some(writer_properties);
    let mut bytes = Vec::new();
    stac::geoparquet::to_writer_with_options(&mut bytes, value, &options)?;
    Ok(bytes.into())
}

impl From<stac::Value> for Value {
    fn from(value: stac::Value) -> Self {
        Value::Stac(value)
    }
}

impl From<stac_api::Item> for Value {
    fn from(value: stac_api::Item) -> Self {
        Self::Json(value.into())
    }
}

impl From<serde_json::Value> for Value {
    fn from(value: serde_json::Value) -> Self {
        Self::Json(value)
    }
}

impl TryFrom<Value> for stac::Value {
    type Error = Error;
    fn try_from(value: Value) -> Result<Self> {
        match value {
            Value::Stac(value) => Ok(value),
            Value::Json(value) => serde_json::from_value(value).map_err(Error::from),
        }
    }
}
