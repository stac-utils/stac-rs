# stac-rs

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gadomski/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac?style=for-the-badge)](https://docs.rs/stac/latest/stac/)
[![Crates.io](https://img.shields.io/crates/v/stac?style=for-the-badge)](https://crates.io/crates/stac)
![Crates.io](https://img.shields.io/crates/l/stac?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.

## Usage

To use the library in your project:

```toml
[dependencies]
stac = "0.3"
```

### Features

There are two opt-in features: `jsonschema` and `reqwest`.

#### jsonschema

The `jsonschema` feature enables validation against [json-schema](https://json-schema.org/) definitions:

```toml
[dependencies]
stac = { version = "0.3", features = ["jsonschema"]}
```

The `jsonschema` feature also enables the `reqwest` feature.

#### reqwest

If you'd like to use the library with `reqwest` for blocking remote reads:

```toml
[dependencies]
stac = { version = "0.3", features = ["reqwest"]}
```

If `reqwest` is not enabled, `stac::read` will throw an error if you try to read from a url.

## Examples

```rust
// Create an item from scratch.
let item = stac::Item::new("an-id");

// Read an item from the filesystem.
let value = stac::read("data/simple-item.json").unwrap();
let item: stac::Item = value.try_into().unwrap();
```

Please see the [documentation](https://docs.rs/stac) for more usage examples.
