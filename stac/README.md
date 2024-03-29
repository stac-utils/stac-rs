# stac

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac?style=for-the-badge)](https://docs.rs/stac/latest/stac/)
[![Crates.io](https://img.shields.io/crates/v/stac?style=for-the-badge)](https://crates.io/crates/stac)
![Crates.io](https://img.shields.io/crates/l/stac?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.

## Usage

To use the library in your project:

```toml
[dependencies]
stac = "0.5"
```

## Examples

```rust
use stac::Item;

// Creates an item from scratch.
let item = Item::new("an-id");

// Reads an item from the filesystem.
let item: Item = stac::read("data/simple-item.json").unwrap();
```

Please see the [documentation](https://docs.rs/stac) for more usage examples.

## Features

There are three opt-in features.

### reqwest

`reqwest` enables blocking remote reads:

```toml
[dependencies]
stac = { version = "0.5", features = ["reqwest"]}
```

Then:

```rust
let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json";
#[cfg(feature = "reqwest")]
let item: stac::Item = stac::read(href).unwrap();
```

If `reqwest` is not enabled, `stac::read` will throw an error if you try to read from a url.

```rust
let href = "https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json";
#[cfg(not(feature = "reqwest"))]
let err = stac::read::<stac::Item>(href).unwrap_err();
```

For non-blocking IO, use the [**stac-async**](https://crates.io/crates/stac-async) crate.

### geo

To use [geojson](https://docs.rs/geojson) and [geo](https://docs.rs/geo) to add some extra geo-enabled methods:

```toml
[dependencies]
stac = { version = "0.5", features = ["geo"] }
```

Then, you can set an item's geometry and bounding box at the same time:

```rust
#[cfg(feature = "geo")]
{
    use stac::Item;
    use geojson::{Geometry, Value};

    let geometry = Geometry::new(Value::Point(vec![
        -105.1, 41.1,
    ]));
    let mut item = Item::new("an-id");

    item.set_geometry(geometry).unwrap();
    assert!(item.bbox.is_some());
}
```

### schemars

`schemars` allows for [jsonschema](https://json-schema.org/) generation from STAC objects.
This is mostly useful for auto-generating OpenAPI documentation for STAC APIs.

```toml
[dependencies]
stac = { version = "0.5", features = ["schemars"]}
```
