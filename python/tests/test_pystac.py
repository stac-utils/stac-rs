import pytest

_ = pytest.importorskip("pystac")

from pathlib import Path  # noqa

import stacrs.pystac  # noqa
from pystac import Item  # noqa


@pytest.fixture
def item(root: Path) -> Item:
    return Item.from_file(root / "python" / "examples" / "simple-item.json")


def test_search() -> None:
    items = stacrs.pystac.search(
        "https://landsatlook.usgs.gov/stac-server",
        collections="landsat-c2l2-sr",
        intersects={"type": "Point", "coordinates": [-105.119, 40.173]},
        sortby="-properties.datetime",
        max_items=1,
    )
    assert len(list(items)) == 1


def test_validate(item: Item) -> None:
    stacrs.pystac.validate(item)


def test_migrate(spec_examples: Path) -> None:
    item = Item.from_file(spec_examples / "v1.0.0" / "simple-item.json")
    d = stacrs.pystac.migrate(item)
    assert d["stac_version"] == "1.1.0"


def test_write(tmp_path: Path, item: Item) -> None:
    path = tmp_path / "out.ndjson"
    stacrs.pystac.write(str(path), [item])


def test_read(examples: Path, root: Path) -> None:
    stacrs.pystac.read(str(examples / "simple-item.json"))
    stacrs.pystac.read(str(root / "core" / "data" / "extended-item.parquet"))
