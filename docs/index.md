# rustac

![Ferris holding STAC](./img/ferris-holding-stac-small.png)

Welcome to the home of STAC and Rust.
We're happy you're here.

## What is rustac?

**rustac** is a [Github repository](https://github.com/stac-utils/rustac) that holds the code for several Rust [crates](https://doc.rust-lang.org/book/ch07-01-packages-and-crates.html) that can be used to create, search for, and otherwise work with [STAC](https://stacspec.org).

## What is stacrs?

**stacrs** (notice the lack of a hyphen) is a Python [package](https://pypi.org/project/stacrs/) that provides a simple API for interacting with STAC.
**stacrs** uses the Rust code in **rustac** under the hood.

```python
import stacrs

items = stacrs.search("s3://bucket/items.parquet", ...)
```

Check out the [stacrs docs](https://stac-utils.github.io/stacrs) for more.

## Why are the names so confusing?

Because [@gadomski](https://github.com/gadomski/), who built and maintains these tools, is really bad at naming stuff.

## Why are rustac and stacrs in two separate repos?

Couple of reasons:

1. **rustac** is intended to be useful on its own.
   It's not just the engine for some Python bindings.
2. Care-and-feeding for Python wheels built from Rust is a bit finicky.
   By moving **stacrs** to its own repo, we're able to separate the concerns of keeping a good, clean Rust core, and building Python wheels.
   Not everyone agrees with this strategy, but here we are.

## Rust documentation on docs.rs

- [stac](https://docs.rs/stac): The core Rust crate
- [stac-api](https://docs.rs/stac-api): Data structures for a STAC API, and a client for searching one
- [stac-server](https://docs.rs/stac-server): A STAC API server with multiple backends
- [pgstac](https://docs.rs/pgstac): Rust bindings for [pgstac](https://github.com/stac-utils/pgstac)
