# stac-rs

Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.

## Usage

We are [**stac**](https://crates.io/crates/stac) on crates.io.
To use the library in your project:

```toml
[dependencies]
stac = "0.3"
```

Please see the [documentation](https://docs.rs/stac) for usage examples.

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
