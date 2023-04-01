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
stac = "0.4"
```

### Features

There are three opt-in features: `jsonschema`, `reqwest`, and `set_query`.

#### jsonschema

The `jsonschema` feature enables validation against [json-schema](https://json-schema.org/) definitions:

```toml
[dependencies]
stac = { version = "0.4", features = ["jsonschema"]}
```

The `jsonschema` feature also enables the `reqwest` feature.

#### reqwest

If you'd like to use the library with `reqwest` for blocking remote reads:

```toml
[dependencies]
stac = { version = "0.4", features = ["reqwest"]}
```

If `reqwest` is not enabled, `stac::read` will throw an error if you try to read from a url.

#### set_query

The `set_query` feature adds a single method to `Link`.
It is behind a feature because it adds a dependency, [serde_urlencoded](https://crates.io/crates/serde_urlencoded).
To enable:

```toml
stac = { version = "0.4", features = ["set_query"]}
```

## Examples

```rust
// Create an item from scratch.
let item = stac::Item::new("an-id");

// Read an item from the filesystem.
let item: stac::Item = stac::read("data/simple-item.json").unwrap();
```

Please see the [documentation](https://docs.rs/stac) for more usage examples.
