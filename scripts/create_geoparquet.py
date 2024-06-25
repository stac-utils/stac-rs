import stac_geoparquet
import json
from pathlib import Path
from pyarrow import Table
from pystac_client import Client

root = Path(__file__).parents[1]
examples = root / "stac-arrow" / "examples"
if not examples.is_dir():
    examples.mkdir(parents=True)

client = Client.open("https://planetarycomputer.microsoft.com/api/stac/v1")
item_search = client.search(
    max_items=10,
    collections=["sentinel-2-l2a"],
    intersects={
        "type": "Point",
        "coordinates": [-105.10, 40.17],
        "sortby": "-properties.datetime",
    },
)
items = list(item_search.items_as_dicts())
batches = stac_geoparquet.arrow.parse_stac_items_to_arrow(items)
table = Table.from_batches(batches)
for version in ["1.0.0", "1.1.0"]:
    path = examples / f"sentinel-2-l2a-{version}.parquet"
    stac_geoparquet.arrow.to_parquet(table, path, schema_version=version)
with open(examples / "sentinel-2-l2a.json", "w") as f:
    json.dump(items, f)
