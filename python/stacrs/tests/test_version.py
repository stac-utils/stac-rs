import stacrs


def test_version() -> None:
    stacrs.version()
    stacrs.version("stac")
    stacrs.version("stac-api")
    stacrs.version("stac-duckdb")
    stacrs.version("duckdb")
