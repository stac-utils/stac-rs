# stac-rs

[![CI](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)

Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.
Documentation are available [here](https://www.gadom.ski/stac-rs/stac/index.html) until this crate is published to crates.io.

## Quickstart

Read STAC objects:

```rust
let object = stac::read("data/catalog.json").unwrap();
```

Read STAC catalogs as trees:

```rust
use stac::Stac;
let (stac, _) = Stac::read("data/catalog.json").unwrap();
```

Write STAC catalogs using the `BestPracticesRenderer`:

```rust
use stac::{BestPracticesRenderer, Render, Writer, Write};
let renderer = BestPracticesRenderer::new("a/new/root/directory").unwrap();
let writer = Writer::default();
stac.write(&renderer, &writer).unwrap();
```

For a more complete walkthrough, see [the documentation](https://www.gadom.ski/stac-rs/stac/index.html).
