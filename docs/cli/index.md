---
description: The stac-rs command-line interface (CLI), stacrs
---

# Command-line interface (CLI)

The **stac-rs** command-line interface can be installed two ways.
If you have Rust, use `cargo`:

```sh
cargo install stac-cli -F duckdb  # to use libduckdb on your system
# or
cargo install stac-cli -F duckdb-bundled  # to build libduckdb on install (slow)
```

The CLI is called **stacrs**:

```shell
stacrs --help
```

If you don't have DuckDB on your system, you can also use the Python wheel, which includes **libduckdb**:

```shell
python -m pip install stacrs
```

For examples of using the CLI, check out the slides from [@gadomski's](https://github.com/gadomski/) 2024 FOSS4G-NA presentation [here](https://www.gadom.ski/2024-09-FOSS4G-NA-stac-rs/).
