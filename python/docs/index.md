# stacrs

A small, no-dependency Python library for working with [STAC](https://stacspec.org), using [Rust](https://github.com/stac-utils/stac-rs) under the hood.

## Installation

```shell
pip install stacrs
```

## Usage

```python
import stacrs

stacrs.validate_href("https://raw.githubusercontent.com/radiantearth/stac-spec/master/examples/simple-item.json")
```

See [the API documentation](./api.md) for more.
