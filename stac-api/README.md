# stac-api

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gadomski/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-api?style=for-the-badge)](https://docs.rs/stac-api/latest/stac_api/)
[![Crates.io](https://img.shields.io/crates/v/stac-api?style=for-the-badge)](https://crates.io/crates/stac-api)
![Crates.io](https://img.shields.io/crates/l/stac-api?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Rust implementation of the data structures that make up the [STAC API specification](https://github.com/radiantearth/stac-api-spec).
This is **not** a server implementation.
For a STAC API server written in Rust, check out [stac-server-rs](https://github.com/gadomski/stac-server-rs).

## Usage

To use the library in your project:

```toml
[dependencies]
stac-api = "0.2"
```

**stac-api** has one optional feature, `schemars`, which can be used to generate [jsonschema](https://json-schema.org/) documents for the API structures.
This is useful for auto-generating OpenAPI documentation:

```toml
[dependencies]
stac-api = { version = "0.2", features = ["schemars"] }
```

## Examples

```rust
use stac_api::{Root, Conformance, CORE_URI};
use stac::Catalog;

// Build the root (landing page) endpoint.
let root = Root {
    catalog: Catalog::new("an-id", "a description"),
    conformance: Conformance {
        conforms_to: vec![CORE_URI.to_string()],
    }
};
```

Please see the [documentation](https://docs.rs/stac-api) for more usage examples.
