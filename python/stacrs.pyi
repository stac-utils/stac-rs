from typing import Any, Optional, Tuple

def migrate_href(href: str, version: Optional[str] = None) -> dict[str, Any]: ...
def migrate(value: dict[str, Any], version: Optional[str] = None) -> dict[str, Any]: ...
def read(
    href: str,
    *,
    format: str | None = None,
    options: list[tuple[str, str]] | None = None,
) -> dict[str, Any]: ...
def search(
    href: str,
    *,
    intersects: Optional[str | dict[str, Any]] = None,
    ids: Optional[str | list[str]] = None,
    collections: Optional[str | list[str]] = None,
    max_items: Optional[int] = None,
    limit: Optional[int] = None,
    bbox: Optional[list[float]] = None,
    datetime: Optional[str] = None,
    include: Optional[str | list[str]] = None,
    exclude: Optional[str | list[str]] = None,
    sortby: Optional[str | list[str]] = None,
    filter: Optional[str | dict[str, Any]] = None,
    query: Optional[dict[str, Any]] = None,
) -> list[dict[str, Any]]: ...
def search_to(
    outfile: str,
    href: str,
    *,
    intersects: Optional[str | dict[str, Any]] = None,
    ids: Optional[str | list[str]] = None,
    collections: Optional[str | list[str]] = None,
    max_items: Optional[int] = None,
    limit: Optional[int] = None,
    bbox: Optional[list[float]] = None,
    datetime: Optional[str] = None,
    include: Optional[str | list[str]] = None,
    exclude: Optional[str | list[str]] = None,
    sortby: Optional[str | list[str]] = None,
    filter: Optional[str | dict[str, Any]] = None,
    query: Optional[dict[str, Any]] = None,
    format: Optional[str] = None,
    options: Optional[list[Tuple[str, str]]] = None,
) -> int: ...
def validate_href(href: str) -> None: ...
def validate(value: dict[str, Any]) -> None: ...
def write(
    href: str,
    value: dict[str, Any] | list[dict[str, Any]],
    *,
    format: str | None = None,
    options: list[tuple[str, str]] | None = None,
) -> dict[str, str] | None: ...
