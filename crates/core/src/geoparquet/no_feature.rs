use crate::geoparquet::{
    impl_from_geoparquet, impl_into_geoparquet, FromGeoparquet, IntoGeoparquet,
};
use bytes::Bytes;
use std::{
    fmt::{Display, Formatter, Result},
    io::Write,
};

/// A dummy unit structure to represent parquet compression when the `geoparquet` feature is not enabled.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Compression;

impl_from_geoparquet!(crate::ItemCollection);
impl_from_geoparquet!(crate::Value);
impl_into_geoparquet!(crate::Item);
impl_into_geoparquet!(crate::ItemCollection);
impl_into_geoparquet!(crate::Value);
impl_into_geoparquet!(serde_json::Value);

impl Display for Compression {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result {
        f.write_str("unspecified compression, geoparquet feature is not enabled")
    }
}
