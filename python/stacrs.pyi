from typing import Any, Optional, Tuple

def migrate_href(href: str, version: Optional[str] = None) -> dict[str, Any]:
    """
    Migrates a STAC dictionary at the given href to another version.

    Migration can be as simple as updating the `stac_version` attribute, but
    sometimes can be more complicated. For example, when migrating to v1.1.0,
    [eo:bands and raster:bands should be consolidated to the new bands
    structure](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0-beta.1).

    See [the stac-rs
    documentation](https://docs.rs/stac/latest/stac/enum.Version.html) for
    supported versions.

    Args:
        href (str): The href to read the STAC object from
        version (str | None): The version to migrate to. If not provided, the
            value will be migrated to the latest stable version.

    Examples:
        >>> item = stacrs.migrate_href("examples/simple-item.json", "1.1.0-beta.1")
        >>> assert item["stac_version"] == "1.1.0-beta.1"
    """

def migrate(value: dict[str, Any], version: Optional[str] = None) -> dict[str, Any]:
    """
    Migrates a STAC dictionary to another version.

    Migration can be as simple as updating the `stac_version` attribute, but
    sometimes can be more complicated. For example, when migrating to v1.1.0,
    [eo:bands and raster:bands should be consolidated to the new bands
    structure](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0-beta.1).

    See [the stac-rs
    documentation](https://docs.rs/stac/latest/stac/enum.Version.html) for
    supported versions.

    Args:
        value (dict[str, Any]): The STAC value to migrate
        version (str | None): The version to migrate to. If not provided, the
            value will be migrated to the latest stable version.

    Returns:
        dict[str, Any]: The migrated dictionary

    Examples:
        >>> with open("examples/simple-item.json") as f:
        >>>     item = json.load(f)
        >>> item = stacrs.migrate(item, "1.1.0-beta.1")
        >>> assert item["stac_version"] == "1.1.0-beta.1"
    """

def read(
    href: str,
    *,
    format: str | None = None,
    options: list[tuple[str, str]] | None = None,
) -> dict[str, Any]:
    """
    Reads STAC from a href.

    Args:
        href (str): The href to write to
        format (str | None): The output format to write. If not provided, will be
            inferred from the href's extension.
        options (list[tuple[str, str]] | None): Options for configuring an
            object store, e.g. your AWS credentials.

    Returns:
        dict[str, Any]: The STAC value

    Examples:
        >>> item = stacrs.read("item.json")
    """

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
) -> list[dict[str, Any]]:
    """
    Searches a STAC API server.

    Args:
        href (str): The STAC API to search.
        intersects (str | dict[str, Any] | GeoInterface | None): Searches items
            by performing intersection between their geometry and provided GeoJSON
            geometry.
        ids (list[str] | None): Array of Item ids to return.
        collections (list[str] | None): Array of one or more Collection IDs that
            each matching Item must be in.
        max_items (int | None): The maximum number of items to iterate through.
        limit (int | None): The page size returned from the server. Use
            `max_items` to actually limit the number of items returned from this
            function.
        bbox (list[float] | None): Requested bounding box.
        datetime (str | None): Single date+time, or a range (`/` separator),
            formatted to RFC 3339, section 5.6.  Use double dots .. for open
            date ranges.
        include (list[str]] | None): fields to include in the response (see [the
            extension
            docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
            for more on the semantics).
        exclude (list[str]] | None): fields to exclude from the response (see [the
            extension
            docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
            for more on the semantics).
        sortby (list[str] | None): Fields by which to sort results (use `-field` to sort descending).
        filter (str | dict[str, Any] | none): CQL2 filter expression. Strings
            will be interpreted as cql2-text, dictionaries as cql2-json.
        query (dict[str, Any] | None): Additional filtering based on properties.
            It is recommended to use filter instead, if possible.
        use_duckdb (bool | None): Query with DuckDB. If None and the href has a
            'parquet' or 'geoparquet' extension, will be set to True. Defaults
            to None.

    Returns:
        list[dict[str, Any]]: A list of the returned STAC items.

    Examples:
        >>> items = stacrs.search(
        ...     "https://landsatlook.usgs.gov/stac-server",
        ...     collections=["landsat-c2l2-sr"],
        ...     intersects={"type": "Point", "coordinates": [-105.119, 40.173]},
        ...     sortby="-properties.datetime",
        ...     max_items=1,
        ... )
    """

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
) -> int:
    """
    Searches a STAC API server and saves the result to an output file.

    Args:
        outfile (str): The output href. This can be a local file path, or any
            url scheme supported by [stac::object_store::write].
        href (str): The STAC API to search.
        intersects (str | dict[str, Any] | GeoInterface | None): Searches items
            by performing intersection between their geometry and provided GeoJSON
            geometry.
        ids (list[str] | None): Array of Item ids to return.
        collections (list[str] | None): Array of one or more Collection IDs that
            each matching Item must be in.
        max_items (int | None): The maximum number of items to iterate through.
        limit (int | None): The page size returned from the server. Use
            `max_items` to actually limit the number of items returned from this
            function.
        bbox (list[float] | None): Requested bounding box.
        datetime (str | None): Single date+time, or a range ('/' separator),
            formatted to RFC 3339, section 5.6.  Use double dots .. for open
            date ranges.
        include (list[str]] | None): fields to include in the response (see [the
            extension
            docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
            for more on the semantics).
        exclude (list[str]] | None): fields to exclude from the response (see [the
            extension
            docs](https://github.com/stac-api-extensions/fields?tab=readme-ov-file#includeexclude-semantics))
            for more on the semantics).
        sortby (list[str] | None): Fields by which to sort results (use `-field` to sort descending).
        filter (str | dict[str, Any] | none): CQL2 filter expression. Strings
            will be interpreted as cql2-text, dictionaries as cql2-json.
        query (dict[str, Any] | None): Additional filtering based on properties.
            It is recommended to use filter instead, if possible.
        format (str | None): The output format. If none, will be inferred from
            the outfile extension, and if that fails will fall back to compact JSON.
        options (list[tuple[str, str]] | None): Configuration values to pass to the object store backend.
        use_duckdb (bool | None): Query with DuckDB. If None and the href has a
            'parquet' or 'geoparquet' extension, will be set to True. Defaults
            to None.

    Returns:
        list[dict[str, Any]]: A list of the returned STAC items.

    Examples:
        >>> items = stacrs.search_to("out.parquet",
        ...     "https://landsatlook.usgs.gov/stac-server",
        ...     collections=["landsat-c2l2-sr"],
        ...     intersects={"type": "Point", "coordinates": [-105.119, 40.173]},
        ...     sortby="-properties.datetime",
        ...     max_items=1,
        ... )
    """

def validate_href(href: str) -> None:
    """
    Validates a STAC dictionary with json-schema.

    Args:
        value (dict[str, Any]): The STAC value to validate

    Raises:
        Exception: On a validation error

    Examples:
        >>> with open("examples/simple-item.json") as f:
        >>>     data = json.load(f)
        >>> stacrs.validate(data)
    """

def validate(value: dict[str, Any]) -> None:
    """
    Validates a single href with json-schema.

    Args:
        href (str): The href of the STAC value to validate

    Raises:
        Exception: On any input/output error, or on a validation error

    Examples:
        >>> stacrs.validate_href("examples/simple-item.json")
        >>> stacrs.validate_href("data/invalid-item.json")
        Traceback (most recent call last):
        File "<stdin>", line 1, in <module>
        Exception: Validation errors: "collection" is a required property
    """

def write(
    href: str,
    value: dict[str, Any] | list[dict[str, Any]],
    *,
    format: str | None = None,
    options: list[tuple[str, str]] | None = None,
) -> dict[str, str] | None:
    """
    Writes STAC to a href.

    Args:
        href (str): The href to write to
        value (dict[str, Any] | list[dict[str, Any]]): The value to write. This
            can be a STAC dictionary or a list of items.
        format (str | None): The output format to write. If not provided, will be
            inferred from the href's extension.
        options (list[tuple[str, str]] | None): Options for configuring an
            object store, e.g. your AWS credentials.

    Returns:
        dict[str, str] | None: The result of putting data into an object store,
            e.g. the e_tag and the version. None is returned if the file was written
            locally.

    Examples:
        >>> with open("items.json") as f:
        ...     items = json.load(f)
        >>> stacrs.write("items.parquet", items)
    """
