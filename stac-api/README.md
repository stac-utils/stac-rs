# stac-api

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gadomski/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-api?style=for-the-badge)](https://docs.rs/stac-api/latest/stac_api/)
[![Crates.io](https://img.shields.io/crates/v/stac-api?style=for-the-badge)](https://crates.io/crates/stac-api)
![Crates.io](https://img.shields.io/crates/l/stac-api?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Rust implementation of the data structures that make up the [STAC API specification](https://github.com/radiantearth/stac-api-spec).
This is **not** a server implementation.
For an (experimental) STAC API server written in Rust, check out [stac-server-rs](https://github.com/gadomski/stac-server-rs).

## Usage

To use the library in your project:

```toml
[dependencies]
stac-api = "0.2"
```

## Examples

```rust
// Build the root (landing page) endpoint.
let root = stac_api::Root {
    catalog: stac::Catalog::new("an-id", "a description"),
    conformsTo: vec!["https://api.stacspec.org/v1.0.0-rc.2/core".to_string()],
};
```

Please see the [documentation](https://docs.rs/stac-api) for more usage examples.
