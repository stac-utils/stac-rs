use crate::Result;
use geoarrow::io::parquet::ParquetReaderOptions;
use stac::Item;
use std::{fs::File, path::Path};

/// Reads a [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet) file from a path.
///
/// # Examples
///
/// ```
/// let items = stac_arrow::read_parquet("examples/sentinel-2-l2a-1.0.0.parquet").unwrap();
/// ```
pub fn read_parquet(path: impl AsRef<Path>) -> Result<Vec<Item>> {
    let file = File::open(path)?;
    let options = ParquetReaderOptions::default(); // TODO make this configurable via a builder
    let table = geoarrow::io::parquet::read_geoparquet(file, options)?;
    crate::table_to_items(table)
}

#[cfg(test)]
mod tests {
    #[test]
    #[cfg(feature = "parquet-compression")]
    fn read_parquet_1_0_0() {
        let items = super::read_parquet("examples/sentinel-2-l2a-1.0.0.parquet").unwrap();
        assert_eq!(items.len(), 10);
    }

    #[test]
    #[cfg(feature = "parquet-compression")]
    fn read_parquet_1_1_0() {
        let items = super::read_parquet("examples/sentinel-2-l2a-1.1.0.parquet").unwrap();
        assert_eq!(items.len(), 10);
    }
}
