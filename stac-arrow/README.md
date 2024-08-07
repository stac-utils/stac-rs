# stac-arrow

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-arrow?style=for-the-badge)](https://docs.rs/stac-arrow/latest/stac_arrow/)
[![Crates.io](https://img.shields.io/crates/v/stac-arrow?style=for-the-badge)](https://crates.io/crates/stac-arrow)
![Crates.io](https://img.shields.io/crates/l/stac-arrow?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Convert STAC item collections to and from [geoarrow](https://github.com/geoarrow/geoarrow-rs/) tables.
To read and write [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet), use [our crate with the same name](../stac-geoparquet/).

**WARNING**: This library should be considered experimental while [geoarrow-rs](https://github.com/geoarrow/geoarrow-rs/) stabilizes.

## Usage

To use the library in your project:

```toml
[dependencies]
stac-arrow = "0.0.1"
```

## Examples

```rust
let item = stac::read("data/simple-item.json").unwrap();
let table = stac_arrow::to_table(vec![item].into()).unwrap();
let item_collection = stac_arrow::from_table(table).unwrap();
```

Please see the [documentation](https://docs.rs/stac-arrow) for more usage examples.

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
