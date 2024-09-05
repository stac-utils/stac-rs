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
stac = "0.9"
```

## Examples

```rust
use stac::Item;

// Creates an item from scratch.
let item = Item::new("an-id");

// Reads an item from the filesystem.
let item: Item = stac::read("examples/simple-item.json").unwrap();
```

Please see the [documentation](https://docs.rs/stac) for more usage examples.

## Features

There are five opt-in features:

- `gdal`
- `geo`
- `geoarrow` (experimental)
- `geoparquet` (experimental)
- `reqwest`

### gdal

To use [GDAL](https://gdal.org) to create items with projection and raster band information, you'll need GDAL installed on your system and enable the `gdal` feature.
Then, items created from rasters will include the projection and raster extensions:

```rust
#[cfg(feature = "gdal")]
{
    use stac::{extensions::{Raster, Projection}, Extensions, item::Builder};
    let item = Builder::new("an-id").asset("data", "assets/dataset_geo.tif").build().unwrap();
    assert!(item.has_extension::<Projection>());
    assert!(item.has_extension::<Raster>());
}
```

### geo

Use [geo](https://docs.rs/geo) to add some extra geo-enabled methods.
For example, you can set an item's geometry and bounding box at the same time:

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

### geoarrow

**⚠️ geoarrow support is currently experimental, and may break on any release.**

[geoarrow](https://geoarrow.org/) is a specification for storing geospatial data in [Apache Arrow](https://arrow.apache.org/) data structures.
It's support in **stac-rs** is currently experimental:

```rust
#[cfg(feature = "geoarrow")]
{
    use stac::ItemCollection;

    let item = stac::read("examples/simple-item.json").unwrap();
    let item_collection = ItemCollection::from(vec![item]);
    let table = stac::geoarrow::to_table(item_collection).unwrap();
}
```

### geoparquet

**⚠️ geoparquet support is currently experimental, and may break on any release.**

[geoparquet](https://geoparquet.org/) is a specification for storing geospatial data in [Apache Parquet](https://parquet.apache.org/) data structures.
It's support in **stac-rs** is currently experimental:

```rust
#[cfg(feature = "geoparquet")]
{
    use std::fs::File;

    let file = File::open("data/extended-item.parquet").unwrap();
    let item_collection = stac::geoparquet::from_reader(file).unwrap();
}
```

### reqwest

`reqwest` enables blocking remote reads:

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

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
