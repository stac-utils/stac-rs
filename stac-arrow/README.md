# stac-arrow

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-arrow?style=for-the-badge)](https://docs.rs/stac-arrow/latest/stac_arrow/)
[![Crates.io](https://img.shields.io/crates/v/stac-arrow?style=for-the-badge)](https://crates.io/crates/stac-arrow)
![Crates.io](https://img.shields.io/crates/l/stac-arrow?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Read and write [STAC](https://stacspec.org/) data stored in [arrow](https://arrow.apache.org/).
Data are formatted per the [stac-geoparquet spec](https://github.com/stac-utils/stac-geoparquet/blob/main/spec/stac-geoparquet-spec.md).

## Usage

To use the library in your project:

```toml
[dependencies]
stac-arrow = "0.1"
```

## Examples

Reading from a [geoparquet](https://geoparquet.org/) file:

```rust
use std::fs::File;
use parquet::arrow::arrow_reader::ParquetRecordBatchReaderBuilder;

let file = File::open("data/naip.parquet").unwrap();
let reader = ParquetRecordBatchReaderBuilder::try_new(file)
    .unwrap()
    .build()
    .unwrap();
let mut items = Vec::new();
for result in reader {
    items.extend(stac_arrow::record_batch_to_items(result.unwrap()).unwrap());
}
assert_eq!(items.len(), 5);
```

Writing:

```rust
use stac::ItemCollection;
use std::io::Cursor;

let item_collection: ItemCollection = stac::read_json("data/naip.json").unwrap();
let mut geo_table = stac_arrow::items_to_geo_table(item_collection.items).unwrap();
let mut cursor = Cursor::new(Vec::new());
geoarrow::io::parquet::write_geoparquet(&mut geo_table, &mut cursor, None).unwrap();
```

Please see the [documentation](https://docs.rs/stac-arrow) for more usage examples.

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
