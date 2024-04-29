# stac-async

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-async?style=for-the-badge)](https://docs.rs/stac-async/latest/stac_async/)
[![Crates.io](https://img.shields.io/crates/v/stac-async?style=for-the-badge)](https://crates.io/crates/stac-async)
![Crates.io](https://img.shields.io/crates/l/stac-async?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Asynchronous I/O for the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.

## Usage

```toml
[dependencies]
stac-async = "0.5"
```

## Examples

```rust
// Read an item.
let url = "https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json";
let value: stac::Item = tokio_test::block_on(async {
    stac_async::read(url).await.unwrap()
});
```

Please see the [documentation](https://docs.rs/stac-async) for more usage examples.

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
