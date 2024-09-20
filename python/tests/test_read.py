from pathlib import Path

import stacrs


def test_read(examples: Path) -> None:
    stacrs.read(str(examples / "simple-item.json"))
