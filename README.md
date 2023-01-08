# stac-rs

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gadomski/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)
![Crates.io](https://img.shields.io/crates/l/stac?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.

<p align="center">
<img src="https://d33wubrfki0l68.cloudfront.net/22691a3c3002324451ed99f4009de8aab761e1b7/d24da/public/images-original/stac-01.png" height="100">
<img src="https://rustacean.net/assets/rustacean-orig-noshadow.svg" height=100>
</p>

## stac: data structures and synchronous I/O

[![docs.rs](https://img.shields.io/docsrs/stac?style=for-the-badge)](https://docs.rs/stac/latest/stac/)
[![Crates.io](https://img.shields.io/crates/v/stac?style=for-the-badge)](https://crates.io/crates/stac)

### Usage

In your `Cargo.toml`:

```toml
[dependencies]
stac = "0.3"
```

See [the README](./stac/README.md) and [the documentation](https://docs.rs/stac) for more.

## stac-async: asynchronous I/O

[![docs.rs](https://img.shields.io/docsrs/stac-async?style=for-the-badge)](https://docs.rs/stac-async/latest/stac_async/)
[![Crates.io](https://img.shields.io/crates/v/stac-async?style=for-the-badge)](https://crates.io/crates/stac-async)

### Usage

In your `Cargo.toml`:

```toml
[dependencies]
stac = "0.3"
stac-async = "0.3"
```

See [the README](./stac-async/README.md) and [the documentation](https://docs.rs/stac-async) for more.

## Development

See [CONTRIBUTING.md](./CONTRIBUTING.md) for information about contributing to this project.
Use [RELEASING.md](./RELEASING.md) as an alternate pull request template when releasing a new version.

## Ecosystem

We have a growing suite of projects in the Rust+STAC ecosystem:

- [pgstac-rs](https://github.com/gadomski/pgstac-rs): Rust interface for [pgstac](https://github.com/stac-utils/pgstac), PostgreSQL schema and functions for STAC
- [stac-server-rs](https://github.com/gadomski/stac-server-rs): A STAC API server implementation
- [stac-incubator-rs](https://github.com/gadomski/stac-incubator-rs): Fledgling projects not yet ready to live on their own in a standalone repo
- [pc-rs](https://github.com/gadomski/pc-rs): Small command line utility for downloading assets from the [Planetary Computer](https://planetarycomputer.microsoft.com/)

## License

**stac-rs** is dual-licensed under both the MIT license and the Apache license (Version 2.0).
See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) for details.
