from typing import Any

import stacrs


def test_migrate(item: dict[str, Any]) -> None:
    item = stacrs.migrate(item, version="1.1.0-beta.1")
    assert item["stac_version"] == "1.1.0-beta.1"
