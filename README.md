# stac-rs

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
![Crates.io](https://img.shields.io/crates/l/stac?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

![Ferris the crab holding the STAC logo](./img/ferris-holding-stac-small.png)

Command Line Interface (CLI) and Rust libraries for the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.

## Formats

**stac-rs** "speaks" three forms of STAC:

- **JSON**: STAC is derived from [GeoJSON](https://geojson.org/)
- **Newline-delimited JSON (ndjson)**: One JSON [item](https://github.com/radiantearth/stac-spec/blob/master/item-spec/item-spec.md) per line, often used for bulk item loading and storage
- **stac-geoparquet**: A newer [specification](https://github.com/stac-utils/stac-geoparquet) for storing STAC items, and optionally collections

We also have interfaces to other storage backends, e.g. Postgres via [pgstac](https://github.com/stac-utils/pgstac).

## Command line interface

Our command line interface (CLI) can query STAC APIs, validate STAC, and more.
Install:

```shell
pip install stacrs-cli
# or
cargo install stac-cli
```

Then:

```shell
# Search
$ stacrs search https://landsatlook.usgs.gov/stac-server \
    --collections landsat-c2l2-sr \
    --intersects '{"type": "Point", "coordinates": [-105.119, 40.173]}' \
    --sortby='-properties.datetime' \
    --max-items 1000 \
    items.parquet

# Translate formats
$ stacrs translate items.parquet items.ndjson
$ stacrs translate items.ndjson items.json

# Migrate STAC versions
$ stacrs translate item-v1.0.json item-v1.1.json --migrate

# Search stac-geoparquet (no API server required)
$ stacrs search items.parquet

# Server
$ stacrs serve items.parquet  # Opens a STAC API server on http://localhost:7822

# Validate
$ stacrs validate item.json
```

## Python

We have Python packages based on **stac-rs** that live in their own repositories:

- [stacrs](https://github.com/gadomski/stacrs) provides a Python API to **stac-rs**, including
  - Reading and writing [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet)
  - Migrating to [STAC v1.1](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0)
  - [More...](https://www.gadom.ski/posts/stacrs-python-v0-1/)
- [pgstacrs](https://github.com/stac-utils/pgstacrs) is a Python library for working with [pgstac](https://github.com/stac-utils/pgstac)

## Crates

This monorepo contains several crates:

| Crate                                            | Description                                                                                     | Badges                                                                                                                                                                                                                                                                  |
| ------------------------------------------------ | ----------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| [stac](./crates/core/README.md)                  | Core data structures and I/O                                                                    | [![docs.rs](https://img.shields.io/docsrs/stac?style=flat-square)](https://docs.rs/stac/latest/stac/) <br> [![Crates.io](https://img.shields.io/crates/v/stac?style=flat-square)](https://crates.io/crates/stac)                                                        |
| [stac-api](./crates/api/README.md)               | Data structures for the [STAC API](https://github.com/radiantearth/stac-api-spec) specification | [![docs.rs](https://img.shields.io/docsrs/stac-api?style=flat-square)](https://docs.rs/stac-api/latest/stac_api/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-api?style=flat-square)](https://crates.io/crates/stac-api)                                    |
| [stac-extensions](./crates/extensions/README.md) | Basic support for [STAC extensions](https://stac-extensions.github.io/)                         | [![docs.rs](https://img.shields.io/docsrs/stac-extensions?style=flat-square)](https://docs.rs/stac-extensions/latest/stac_extensions/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-extensions?style=flat-square)](https://crates.io/crates/stac-extensions) |
| [stac-cli](./crates/cli/README.md)               | Command line interface                                                                          | [![docs.rs](https://img.shields.io/docsrs/stac-cli?style=flat-square)](https://docs.rs/stac-cli/latest/stac_cli/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-cli?style=flat-square)](https://crates.io/crates/stac-cli)                                    |
| [stac-server](./crates/server/README.md)         | STAC API server with multiple backends                                                          | [![docs.rs](https://img.shields.io/docsrs/stac-server?style=flat-square)](https://docs.rs/stac-server/latest/stac_server/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-server?style=flat-square)](https://crates.io/crates/stac-server)                     |
| [pgstac](./crates/pgstac/README.md)              | Bindings for [pgstac](https://github.com/stac-utils/pgstac)                                     | [![docs.rs](https://img.shields.io/docsrs/pgstac?style=flat-square)](https://docs.rs/pgstac/latest/pgstac/) <br> [![Crates.io](https://img.shields.io/crates/v/pgstac?style=flat-square)](https://crates.io/crates/pgstac)                                              |
| [stac-duckdb](./crates/duckdb/README.md)         | Experimental client for [duckdb](https://duckdb.org/)                                           | [![docs.rs](https://img.shields.io/docsrs/stac-duckdb?style=flat-square)](https://docs.rs/stac-duckdb/latest/stac_duckdb/) <br> [![Crates.io](https://img.shields.io/crates/v/stac-duckdb?style=flat-square)](https://crates.io/crates/stac-duckdb)                     |

## Development

See [CONTRIBUTING.md](./CONTRIBUTING.md) for information about contributing to this project.
See [RELEASING.md](./RELEASING.md) for a checklist to use when releasing a new version.

## License

**stac-rs** is dual-licensed under both the MIT license and the Apache license (Version 2.0).
See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) for details.

<!-- markdownlint-disable-file MD033 -->
