# stac-rs

[![CI](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg)](./CODE_OF_CONDUCT) 

Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.
Not yet published to crates.io.

## Quickstart

Read STAC objects:

```rust
let object = stac::read("data/catalog.json").unwrap();
println!("{}", object.id());
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

## Examples

There is a brief example at [examples/copy.rs](./examples/copy.rs) that demonstrates a simple read-write operation.
You can use it from the command line:

```shell
cargo run --examples copy data/catalog.json tmp
```

## License

**stac-rs** is dual-licenced under both the MIT license and the Apache license (Version 2.0).
See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) for details.
