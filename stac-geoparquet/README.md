# stac

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-geoparquet?style=for-the-badge)](https://docs.rs/stac-geoparquet/latest/stac_geoparquet/)
[![Crates.io](https://img.shields.io/crates/v/stac-geoparquet?style=for-the-badge)](https://crates.io/crates/stac-geoparquet)
![Crates.io](https://img.shields.io/crates/l/stac-geoparquet?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Read and write [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet).

## Usage

To use the library in your project:

```toml
[dependencies]
stac-geoparquet = "0.0.1"
```

## Examples

```rust
use std::{fs::File, io::Cursor};
use stac::Item;

let item: Item = stac::read("data/simple-item.json").unwrap();
let mut cursor = Cursor::new(Vec::new());
stac_geoparquet::to_writer(&mut cursor, item.into()).unwrap();

let file = File::open("examples/extended-item.parquet").unwrap();
let item_collection = stac_geoparquet::from_reader(file).unwrap();
```

Please see the [documentation](https://docs.rs/stac) for more usage examples.

## Features

There is one feature, enabled by default.

### compression

`compression` enables parquet compression, and is enabled by default.
To disable:

```toml
[dependencies]
stac-geoparquet = { version = "0.0.1", default_features = false }
```

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
