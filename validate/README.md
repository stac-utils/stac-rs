# stac-validate

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac-validate?style=for-the-badge)](https://docs.rs/stac-validate/latest/stac-validate/)
[![Crates.io](https://img.shields.io/crates/v/stac-validate?style=for-the-badge)](https://crates.io/crates/stac-validate)
![Crates.io](https://img.shields.io/crates/l/stac-validate?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Validate [STAC](https://stacspec.org/) with [jsonschema](https://json-schema.org/).

## Usage

To use the library in your project:

```toml
[dependencies]
stac-validate = "0.2"
```

## Examples

```rust
use stac_validate::Validate;
let item: stac::Item = stac::read("data/simple-item.json").unwrap();
item.validate().unwrap();
```

Please see the [documentation](https://docs.rs/stac-validate) for more usage examples.

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
