# stac-rs

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/gadomski/stac-rs/CI?style=for-the-badge)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac?style=for-the-badge)](https://docs.rs/stac/latest/stac/)
[![Crates.io](https://img.shields.io/crates/v/stac?style=for-the-badge)](https://crates.io/crates/stac)
![Crates.io](https://img.shields.io/crates/l/stac?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.

## Using the library

We are [**stac**](https://crates.io/crates/stac) on crates.io.
To use the library in your project:

```toml
[dependencies]
stac = "0.1"
```

### Features

There are two opt-in features: `jsonschema` and `reqwest`.

#### jsonschema

The `jsonschema` feature enables validation against [json-schema](https://json-schema.org/) definitions:

```toml
[dependencies]
stac = { version = "0.1", features = ["jsonschema"]}
```

The `jsonschema` feature also enables the `reqwest` feature.

#### reqwest

If you'd like to use the library with `reqwest` for blocking remote reads:

```toml
[dependencies]
stac = { version = "0.1", features = ["reqwest"]}
```

If `reqwest` is not enabled, `stac::read` will throw an error if you try to read from a url.

## API

Please see the [documentation](https://docs.rs/stac/latest/stac/) for usage examples.

## Incubator

We have an [incubator repository](https://github.com/gadomski/stac-rs-incubator) that holds related projects that aren't ready to be released as their own repositories.
These include (or are planned to include):

- command line interface
- STAC-API client
- STAC-API server

## Development

See [CONTRIBUTING.md](./CONTRIBUTING.md) for information about contributing to this project.
Use [RELEASING.md](./RELEASING.md) as an alternate pull request template when releasing a new version.

## License

**stac-rs** is dual-licensed under both the MIT license and the Apache license (Version 2.0).
See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) for details.
