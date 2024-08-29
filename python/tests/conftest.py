import json
from pathlib import Path
from typing import Any

import pytest


@pytest.fixture
def root() -> Path:
    return Path(__file__).parents[2]


@pytest.fixture
def spec_examples(root: Path) -> Path:
    return root / "spec-examples" / "v1.0.0"


@pytest.fixture
def data(root: Path) -> Path:
    return root / "python" / "data"


@pytest.fixture
def item(spec_examples: Path) -> dict[str, Any]:
    with open(spec_examples / "simple-item.json") as f:
        return json.load(f)
