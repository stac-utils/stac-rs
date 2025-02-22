# stac-cli

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-cli?style=for-the-badge)](https://docs.rs/stac-cli/latest/stac_cli/)
[![Crates.io](https://img.shields.io/crates/v/stac-cli?style=for-the-badge)](https://crates.io/crates/stac-cli)
![Crates.io](https://img.shields.io/crates/l/stac-cli?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Command Line Interface (CLI) for [STAC](https://stacspec.org/), named `stacrs`.

## Installation

```sh
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
$ stac search items.parquet

# Server
$ stacrs serve items.parquet  # Opens a STAC API server on http://localhost:7822

# Validate
$ stacrs validate item.json
```

## Usage

**stacrs** provides the following subcommands:

- `stacrs search`: searches STAC APIs and geoparquet files
- `stacrs serve`: serves a STAC API
- `stacrs translate`: converts STAC from one format to another
- `stacrs validate`: validates a STAC value

Use the `--help` flag to see all available options for the CLI and the subcommands:

## Features

This crate has two features:

- `pgstac`: enable a [pgstac](https://github.com/stac-utils/pgstac) backend for `stacrs serve` (enabled by default)
- `duckdb-bundled`: bundle DuckDB by building it from source, instead of using a local installation (enabled by default)

### DuckDB

If you have DuckDB installed locally and want to use that shared library (instead of building it when you install), disable the `duckdb-bundled` feature:

```shell
cargo install stac-cli --no-default-features -F pgstac
```

This can dramatically speed up install time.

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
