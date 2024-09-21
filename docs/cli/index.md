# Command-line interface (CLI)

The **stac-rs** command-line interface can be installed two ways.
If you have Rust, use `cargo`:

```shell
cargo install stac-cli
```

If you have Python, use `pip`:

```shell
pip install stacrs-cli
```

!!! Note
    <!-- markdownlint-disable-next-line MD046 -->
    The PyPI version of the CLI does not contain bindings to GDAL. This
    shouldn't be a problem for most users, but if you're using `stacrs item
    image.tiff` to generate new STAC items from a raster, you'll need to install
    from `cargo`.

The CLI is called **stacrs**:

```shell
stacrs --help
```

For examples of using the CLI, check out the slides from [@gadomski's](https://github.com/gadomski/) 2024 FOSS4G-NA presentation [here](https://www.gadom.ski/2024-09-FOSS4G-NA-stac-rs/).
