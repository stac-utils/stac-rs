---
description: The stac-rs Python package, stacrs
---

# stacrs

A Python package for working with [STAC](https://stacspec.org) designed to compliment existing packages such as [pystac](https://pystac.readthedocs.io) and [pystac-client](https://pystac-client.readthedocs.io).

## Usage

Install via **pip**:

```shell
pip install stacrs
```

Then:

```python
import stacrs

# Search a STAC API
items = stacrs.search(
    "https://landsatlook.usgs.gov/stac-server",
    collections="landsat-c2l2-sr",
    intersects={"type": "Point", "coordinates": [-105.119, 40.173]},
    sortby="-properties.datetime",
    max_items=100,
)

# Write items to a stac-geoparquet file
stacrs.write("items.parquet", items)

# Read items from a stac-geoparquet file as an item collection
item_collection = stacrs.read("items.parquet")

# Use `search_to` for better performance if you know you'll be writing the items
# to a file
stacrs.search_to(
    "items.parquet",
    "https://landsatlook.usgs.gov/stac-server",
    collections="landsat-c2l2-sr",
    intersects={"type": "Point", "coordinates": [-105.119, 40.173]},
    sortby="-properties.datetime",
    max_items=100,
)
```

## Comparisons

This package (intentionally) has limited functionality, as it is _not_ intended to be a replacement for existing Python STAC packages.
[pystac](https://pystac.readthedocs.io) is a mature Python library with a significantly richer API for working with STAC objects.
For querying STAC APIs, [pystac-client](https://pystac-client.readthedocs.io) is more feature-rich than our simplistic `stacrs.search`.

That being said, it is hoped that **stacrs** will be a nice complement to the existing Python STAC ecosystem by providing a no-dependency package with unique capabilities, such as searching directly into a stac-geoparquet file.
