# stac-cli

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/rustac/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/rustac/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-cli?style=for-the-badge)](https://docs.rs/stac-cli/latest/stac_cli/)
[![Crates.io](https://img.shields.io/crates/v/stac-cli?style=for-the-badge)](https://crates.io/crates/stac-cli)
![Crates.io](https://img.shields.io/crates/l/stac-cli?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Command Line Interface (CLI) for [STAC](https://stacspec.org/), named `stacrs`.

## Installation

```sh
cargo install stac-cli -F duckdb  # to use libduckdb on your system
# or
cargo install stac-cli -F duckdb-bundled  # to build libduckdb on install (slow)
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

- `stacrs search`: searches STAC APIs and, if the `duckdb` feature is enabled, geoparquet files
- `stacrs serve`: serves a STAC API
- `stacrs translate`: converts STAC from one format to another
- `stacrs validate`: validates a STAC value

Use the `--help` flag to see all available options for the CLI and the subcommands:

## Features

This crate has three features:

- `pgstac`: enable a [pgstac](https://github.com/stac-utils/pgstac) backend for `stacrs serve`
- `duckdb`: build with DuckDB support, which enables searching [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet) (requires DuckDB to be present on your system)
- `duckdb-bundled`: bundle DuckDB by building it from source, instead of using a local installation (does _not_ require DuckDB to be present on your system)

> [!TIP]
> If you're using the `duckdb` feature, set `DUCKDB_LIB_DIR` to the directory containing your **libduckdb**. If you're on macos and using [Homebrew](https://brew.sh/), this might be `export DUCKDB_LIB_DIR=/opt/homebrew/lib`

## Other info

This crate is part of the [rustac](https://github.com/stac-utils/rustac) monorepo, see its README for contributing and license information.
