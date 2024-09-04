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
pip install stacrs-cli
```

Then:

```shell
stacrs --help
```

**NOTE:** the version from PyPI (installed with `pip`) does not include GDAL support.
If you need to use [gdal](../core/README.md#gdal) features, install via `cargo install`.

## Usage

**stacrs** provides the following subcommands:

- `stacrs item`: create STAC items and combine them into item collections
- `stacrs search`: search STAC APIs
- `stacrs serve`: serve a STAC API
- `stacrs sort`: sort the fields of STAC items, catalogs, and collections
- `stacrs translate`: convert STAC values from one format to another
- `stacrs validate`: validate STAC items, catalogs, and collections using [json-schema](https://json-schema.org/)

Use the `--help` flag to see all available options for the CLI and the subcommands:

## Features

By default, the CLI builds w/ [GDAL](https://gdal.org) support, and will error if GDAL is not installed on your system.
If you don't want to use GDAL, you can disable the default features:

```shell
cargo install stac-cli --no-default-features
```

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
