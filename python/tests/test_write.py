from pathlib import Path
from typing import Any

import pyarrow.parquet
import stac_geoparquet
import stacrs


def test_write(item: dict[str, Any], tmp_path: Path) -> None:
    path = str(tmp_path / "out.parquet")
    stacrs.write(path, [item])
    table = pyarrow.parquet.read_table(path)
    items = list(stac_geoparquet.arrow.stac_table_to_items(table))
    assert len(items) == 1
