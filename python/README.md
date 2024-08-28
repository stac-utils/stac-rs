# stacrs

A Python package for working with [STAC](https://stacspec.org/), using Rust under the hood.

## Usage

Install via **pip**:

```shell
pip install stacrs
```

Then:

```python
import stacrs

stacrs.validate_href("https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json")
```

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
