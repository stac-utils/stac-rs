from pathlib import Path

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
