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

Or:

```shell
# NOTE: The version from PyPI does not include GDAL or DuckDB support. If you
# need to use these features, install via `cargo install` (GDAL is enabled by
# default) or `cargo install -F duckdb` (DuckDB is not).
pip install stacrs-cli
```

Then:

```shell
stacrs --help
```

## Usage

**stacrs** provides the following subcommands:

- `stacrs item`: create STAC items and combine them into item collections
- `stacrs migrate`: migrate a STAC object to another version
- `stacrs search`: search STAC APIs (and geoparquet, with the experimental `duckdb` feature)
- `stacrs serve`: serve a STAC API (optionally, with a [pgstac](https://github.com/stac-utils/pgstac) backend)
- `stacrs translate`: convert STAC values from one format to another
- `stacrs validate`: validate STAC items, catalogs, and collections using [json-schema](https://json-schema.org/)

Use the `--help` flag to see all available options for the CLI and the subcommands:

## Features

This crate has five features, three of them on by default:

- `duckdb`: experimental support for querying [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet) files using [DuckDB](https://duckdb.org/)
- `gdal`: read geospatial data from rasters (enabled by default)
- `geoparquet`: read and write [stac-geoparquet](https://github.com/stac-utils/stac-geoparquet) (enabled by default)
- `pgstac`: enable a [pgstac](https://github.com/stac-utils/pgstac) backend for `stacrs serve` (enabled by default)
- `python`: create an entrypoint that can be called from Python (used to enable `pip install stacrs-cli`)

If you don't want to use GDAL or any of the other default features:

```shell
cargo install stac-cli --no-default-features
```

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
