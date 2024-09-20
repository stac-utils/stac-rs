from pathlib import Path
from typing import Any

import stacrs


def test_migrate(item: dict[str, Any]) -> None:
    item = stacrs.migrate(item, version="1.1.0")
    assert item["stac_version"] == "1.1.0"


def test_migrate_href(spec_examples: Path) -> None:
    item = stacrs.migrate_href(
        str(spec_examples / "v1.0.0" / "simple-item.json"), version="1.1.0"
    )
    assert item["stac_version"] == "1.1.0"
