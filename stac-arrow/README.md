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

let file = File::open("data/naip.parquet").unwrap();
let geo_table = geoarrow::io::parquet::read_geoparquet(file, Default::default()).unwrap();
let items = stac_arrow::geo_table_to_items(geo_table).unwrap();
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
