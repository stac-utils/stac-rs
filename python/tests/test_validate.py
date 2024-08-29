from pathlib import Path
from typing import Any

import pytest
import stacrs
from stacrs import StacrsError


def test_validate_href_ok(spec_examples: Path) -> None:
    stacrs.validate_href(str(spec_examples / "simple-item.json"))


def test_validate_href_invalid(data: Path) -> None:
    with pytest.raises(StacrsError):
        stacrs.validate_href(str(data / "invalid-item.json"))


def test_validate(item: dict[str, Any]) -> None:
    stacrs.validate(item)
