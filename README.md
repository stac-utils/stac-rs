# stac-rs

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
![Crates.io](https://img.shields.io/crates/l/stac?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

![Ferris the crab holding the STAC logo](./img/rustacean-and-stac-small.png)

Command Line Interface (CLI) and Rust libraries for the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.

- Use [stac-cli](./stac-cli/README.md) to query a STAC API, create and validate STAC items, and do other awesome stuff on the command line.
- Use the core [stac](./stac/README.md) library to incorporate STAC data structures (`Item`, `Catalog`, and `Collection`) in another Rust application.
- Use [stac-async](./stac-async/README.md) to build an application that uses async Rust via [tokio](https://tokio.rs/).
- Use [stac-server](./stac-server/README.md) to serve a STAC API

## Crates

This monorepo contains several crates:

| Crate | Description | Badges |
| ----- | ---- | --------- |
| [stac](./stac/README.md) | Core data structures and synchronous I/O | [![docs.rs](https://img.shields.io/docsrs/stac?style=flat-square)](https://docs.rs/stac/latest/stac/) <br> [![Crates.io](https://img.shields.io/crates/v/stac?style=flat-square)](https://crates.io/crates/stac) |
| [pgstac](./pgstac/README.md) | Bindings for [pgstac](https://github.com/stac-utils/pgstac) | [![docs.rs](https://img.shields.io/docsrs/pgstac?style=flat-square)](https://docs.rs/pgstac/latest/pgstac/) <br> [![Crates.io](https://img.shields.io/crates/v/pgstac?style=flat-square)](https://crates.io/crates/pgstac) |
| [stac-api](./stac-api/README.md) | Data structures for the [STAC API](https://github.com/radiantearth/stac-api-spec) specification | [![docs.rs](https://img.shields.io/docsrs/stac-api?style=flat-square)](https://docs.rs/stac-api/latest/stac_api/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-api?style=flat-square)](https://crates.io/crates/stac-api) |
| [stac-async](./stac-async/README.md) | Asynchronous I/O with [tokio](https://tokio.rs/) | [![docs.rs](https://img.shields.io/docsrs/stac-async?style=flat-square)](https://docs.rs/stac-async/latest/stac_async/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-async?style=flat-square)](https://crates.io/crates/stac-async) |
| [stac-cli](./stac-cli/README.md)| Command line interface | [![docs.rs](https://img.shields.io/docsrs/stac-cli?style=flat-square)](https://docs.rs/stac-cli/latest/stac_cli/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-cli?style=flat-square)](https://crates.io/crates/stac-cli) |
| [stac-server](./stac-server/README.md)| STAC API server with multiple backends | [![docs.rs](https://img.shields.io/docsrs/stac-server?style=flat-square)](https://docs.rs/stac-server/latest/stac_server/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-server?style=flat-square)](https://crates.io/crates/stac-server) |
| [stac-validate](./stac-validate/README.md) | Validate STAC data structures with [jsonschema](https://json-schema.org/) | [![docs.rs](https://img.shields.io/docsrs/stac-validate?style=flat-square)](https://docs.rs/stac-validate/latest/stac-validate/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-validate?style=flat-square)](https://crates.io/crates/stac-validate) |

## Bindings

### Python

**stacrs** is a small, no-dependency Python library that uses **stac-rs** under the hood.
Install with **pip**:

```shell
pip install stacrs
```

See [the documentation](https://stacrs.readthedocs.io/) for more information.

## Development

See [CONTRIBUTING.md](./CONTRIBUTING.md) for information about contributing to this project.
See [RELEASING.md](./RELEASING.md) for a checklist to use when releasing a new version.

## License

**stac-rs** is dual-licensed under both the MIT license and the Apache license (Version 2.0).
See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) for details.

<!-- markdownlint-disable-file MD033 -->
