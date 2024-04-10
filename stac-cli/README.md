# stac-cli

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-cli?style=for-the-badge)](https://docs.rs/stac-cli/latest/stac_cli/)
[![Crates.io](https://img.shields.io/crates/v/stac-cli?style=for-the-badge)](https://crates.io/crates/stac-cli)
![Crates.io](https://img.shields.io/crates/l/stac-cli?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Command line interface to [stac-rs](https://github.com/stac-utils/stac-rs).

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

### Item

Create a STAC Item from an href:

```shell
stac item https://storage.googleapis.com/open-cogs/stac-examples/20201211_223832_CS2.tif
```

### Search

Search a STAC API:

```shell
stac search https://earth-search.aws.element84.com/v1 \
    -c sentinel-2-l2a \
    --max-items 1 \
    --sortby='-properties.datetime' \
    --intersects '{"type":"Point","coordinates":[-105.1019,40.1672]}'
```

### Validate

Validate a STAC item:

```shell
stac validate https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json
```

### Sort

Sort the fields of a STAC item into the order they're specified in the spec:

```shell
stac sort https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json
```
