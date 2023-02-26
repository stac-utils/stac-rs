# stac-rs

<!-- Allow html tags -->
<!-- markdownlint-disable MD033 -->

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gadomski/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)
![Crates.io](https://img.shields.io/crates/l/stac?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification, spread over several crates.

<p align="center">
<img src="https://github.com/radiantearth/stac-site/raw/main/assets/images/STAC-01.png" height="100">
<img src="https://rustacean.net/assets/rustacean-orig-noshadow.svg" height=100>
</p>

| Crate | Description | |
| ----- | ---- | --------- |
| **stac** | Core data structures and synchronous I/O | [![README](https://img.shields.io/static/v1?label=README&message=stac&color=informational&style=flat-square)](./stac/README.md) <br> [![docs.rs](https://img.shields.io/docsrs/stac?style=flat-square)](https://docs.rs/stac/latest/stac/) <br> [![Crates.io](https://img.shields.io/crates/v/stac?style=flat-square)](https://crates.io/crates/stac) |
| **stac-api** | Data structures for the [STAC API](https://github.com/radiantearth/stac-api-spec) specification | [![README](https://img.shields.io/static/v1?label=README&message=stac-api&color=informational&style=flat-square)](./stac-api/README.md) <br> [![docs.rs](https://img.shields.io/docsrs/stac-api?style=flat-square)](https://docs.rs/stac-api/latest/stac_api/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-api?style=flat-square)](https://crates.io/crates/stac-api)
| **stac-async** | Asynchronous I/O with [tokio](https://tokio.rs/) | [![README](https://img.shields.io/static/v1?label=README&message=stac-async&color=informational&style=flat-square)](./stac-async/README.md) <br> [![docs.rs](https://img.shields.io/docsrs/stac-async?style=flat-square)](https://docs.rs/stac-async/latest/stac_async/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-async?style=flat-square)](https://crates.io/crates/stac-async)
| **stac-cli** | Command line interface | [![README](https://img.shields.io/static/v1?label=README&message=stac-cli&color=informational&style=flat-square)](./stac-cli/README.md) <br> [![docs.rs](https://img.shields.io/docsrs/stac-cli?style=flat-square)](https://docs.rs/stac-cli/latest/stac_cli/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-cli?style=flat-square)](https://crates.io/crates/stac-cli)

## What are you trying to do?

### Use STAC data structures in your own project

Use [stac](./stac/README.md).
In your `Cargo.toml`:

```toml
[dependencies]
stac = "0.3"
```

Then, in your project:

```rust
use stac::Item;
let item = Item::new("an-id");
```

### Use a command line interface

Install [stac-cli](./stac-cli/README.md) from crates.io:

```shell
cargo install stac-cli
```

See all the subcommands available:

```shell
stac --help
```

### Asynchronously stream STAC objects from a STAC API

Use the `ApiClient` from [stac-async](./stac-async/README.md).
In your `Cargo.toml`:

```toml
[dependencies]
stac-api = "0.1"
stac-async = "0.3"
futures-util = "*"
```

Then, in your project:

```rust
use stac_async::ApiClient;
use stac_api::Search;
use futures_util::stream::StreamExt;

let client = ApiClient::new("https://planetarycomputer.microsoft.com/api/stac/v1").unwrap();
let search = Search {
    collections: Some(vec!["sentinel-2-l2a".to_string()]),
    limit: Some(1),
    ..Default::deafult()
};
let items = Vec<_> = client
    .search(search)
    .await
    .unwrap()
    .map(|result| result.unwrap())
    .collect()
    .await;
```

### Build a STAC API server

Use [stac-api](./stac-api/README.md)

```toml
[dependencies]
stac-api = "0.1"
```

See [stac-server-rs](https://github.com/gadomski/stac-server-rs) for one example of a STAC API server built using these crates.

## Development

See [CONTRIBUTING.md](./CONTRIBUTING.md) for information about contributing to this project.
See [RELEASING.md](./RELEASING.md) for a checklist to use when releasing a new version.

## Ecosystem

We have a growing suite of projects in the Rust+STAC ecosystem:

- [pgstac-rs](https://github.com/gadomski/pgstac-rs): Rust interface for [pgstac](https://github.com/stac-utils/pgstac), PostgreSQL schema and functions for STAC
- [stac-server-rs](https://github.com/gadomski/stac-server-rs): A STAC API server implementation
- [pc-rs](https://github.com/gadomski/pc-rs): Small command line utility for downloading assets from the [Planetary Computer](https://planetarycomputer.microsoft.com/)

## License

**stac-rs** is dual-licensed under both the MIT license and the Apache license (Version 2.0).
See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) for details.
