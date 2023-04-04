# stac-cli

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/gadomski/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-cli?style=for-the-badge)](https://docs.rs/stac-cli/latest/stac_cli/)
[![Crates.io](https://img.shields.io/crates/v/stac-cli?style=for-the-badge)](https://crates.io/crates/stac-cli)
![Crates.io](https://img.shields.io/crates/l/stac-cli?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Command line interface to [stac-rs](https://github.com/gadomski/stac-rs).

## Installation

Install rust.
[rustup](https://rustup.rs/) works well.
Once you do:

```sh
cargo install stac-cli
```

## Usage

Use the cli `--help` flag to see all available options:

```shell
stac --help
```

### Download

Download all the assets of a STAC item.
The STAC item will be written out, with its assets updated to point to the locally downloaded assets.

```shell
stac download https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json .
```
