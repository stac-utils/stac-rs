import json

import stac_geoparquet

with open("stac-arrow/data/naip.json") as f:
    items = json.load(f)["features"]
dataframe = stac_geoparquet.to_geodataframe(items)
dataframe.to_parquet("stac-arrow/data/naip.parquet")
