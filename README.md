# stac-rs

[![GitHub Workflow Status](https://img.shields.io/github/workflow/status/gadomski/stac-rs/CI?style=for-the-badge)](https://github.com/gadomski/stac-rs/actions/workflows/ci.yml)
[![docs.rs](https://img.shields.io/docsrs/stac?style=for-the-badge)](https://docs.rs/stac/latest/stac/)
[![Crates.io](https://img.shields.io/crates/v/stac?style=for-the-badge)](https://crates.io/crates/stac) \
![Crates.io](https://img.shields.io/crates/l/stac?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT) 

Rust implementation of the [SpatioTemporal Asset Catalog (STAC)](https://stacspec.org/) specification.

## Examples

Read STAC objects:

```rust
let object = stac::read("data/catalog.json").unwrap();
println!("{}", object.id());
```

Write STAC catalogs using the `BestPracticesRenderer`:

```rust
use stac::{Stac, BestPracticesRenderer, Render, Writer, Write};
let (stac, _) = Stac::read("data/catalog.json").unwrap();
let renderer = BestPracticesRenderer::new("a/new/root/directory").unwrap();
let writer = Writer::default();
stac.write(&renderer, &writer).unwrap();
```

For more, see the [documentation](https://docs.rs/stac/latest/stac/).

## Executables

As of now, there is no command line interface.
There is an example at [examples/copy.rs](./examples/copy.rs) that demonstrates a simple read-write operation.
To run it from the command line:

```shell
cargo run --examples copy data/catalog.json tmp
```

## Development

See [CONTRIBUTING.md](./CONTRIBUTING.md) for information about contributing to this project.
See [RELEASING.md](./RELEASING.md) for instructions on releasing a new version.

## License

**stac-rs** is dual-licensed under both the MIT license and the Apache license (Version 2.0).
See [LICENSE-APACHE](./LICENSE-APACHE) and [LICENSE-MIT](./LICENSE-MIT) for details.
