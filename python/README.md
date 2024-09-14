# stacrs

[![GitHub Workflow Status](https://img.shields.io/github/actions/workflow/status/stac-utils/stac-rs/ci.yml?branch=main&style=for-the-badge)](https://github.com/stac-utils/stac-rs/actions/workflows/ci.yml)
[![PyPI - Version](https://img.shields.io/pypi/v/stacrs?style=for-the-badge)](https://pypi.org/project/stacrs)
[![Read the Docs](https://img.shields.io/readthedocs/stacrs?style=for-the-badge)](https://stacrs.readthedocs.io/)
![PyPI - License](https://img.shields.io/pypi/l/stacrs?style=for-the-badge)
[![Contributor Covenant](https://img.shields.io/badge/Contributor%20Covenant-2.1-4baaaa.svg?style=for-the-badge)](./CODE_OF_CONDUCT)

A Python package for working with [STAC](https://stacspec.org/), using Rust under the hood.

## Usage

Install via **pip**:

```shell
pip install stacrs
```

Then:

```python
import stacrs

# Searches a STAC API
items = stacrs.search(
    "https://landsatlook.usgs.gov/stac-server",
    collections="landsat-c2l2-sr",
    intersects={"type": "Point", "coordinates": [-105.119, 40.173]},
    sortby="-properties.datetime",
    max_items=1,
)

# Validates a href using json-schema
stacrs.validate_href("https://raw.githubusercontent.com/radiantearth/stac-spec/v1.0.0/examples/simple-item.json")
```

See [the documentation](https://stacrs.readthedocs.io/) for more information.

## Other info

This crate is part of the [stac-rs](https://github.com/stac-utils/stac-rs) monorepo, see its README for contributing and license information.
