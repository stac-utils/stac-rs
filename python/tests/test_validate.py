from pathlib import Path
from typing import Any

import pytest
import stacrs


def test_validate_href_ok(spec_examples: Path) -> None:
    stacrs.validate_href(str(spec_examples / "simple-item.json"))


def test_validate_href_invalid(data: Path) -> None:
    with pytest.raises(Exception):
        stacrs.validate_href(str(data / "invalid-item.json"))


def test_validate(item: dict[str, Any]) -> None:
    stacrs.validate(item)
