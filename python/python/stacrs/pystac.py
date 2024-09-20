from typing import Any, Iterable, Optional

import pystac
from pystac import Catalog, Item, ItemCollection, STACObject

from . import stacrs


def migrate(
    value: STACObject,
    version: Optional[str] = None,
    *,
    include_self_link: bool = True,
    transform_hrefs: bool = False,
) -> dict[str, Any]:
    """
    Migrates a STAC dictionary to another version.

    Because pystac doesn't store the STAC version on the STACObject, this
    function returns a dictionary to ensure the version is preserved.

    Migration can be as simple as updating the `stac_version` attribute, but
    sometimes can be more complicated. For example, when migrating to v1.1.0,
    [eo:bands and raster:bands should be consolidated to the new bands
    structure](https://github.com/radiantearth/stac-spec/releases/tag/v1.1.0-beta.1).

    See [the stac-rs
    documentation](https://docs.rs/stac/latest/stac/enum.Version.html) for
    supported versions.

    Args:
        value (STACObject): The STAC value to migrate
        version (str | None): The version to migrate to. If not provided, the
            value will be migrated to the latest stable version.
        include_self_link (bool): Whether to include a self link when
            serializing this object to a dictionary.
        transform_hrefs (bool): Whether to transform this object's hrefs when
            serializing it to a dictionary. While pystac defaults to true, this
            function defaults to false
            (https://github.com/stac-utils/pystac/issues/960)

    Returns:
        dict[str, Any]: The migrated object

    Examples:
        >>> item = pystac.read_file("old-item.json")
        >>> item = stacrs.pystac.migrate(item, "1.1.0")
        >>> assert item["stac_version"] == "1.1.0"
    """
    return stacrs.migrate(
        value.to_dict(
            include_self_link=include_self_link, transform_hrefs=transform_hrefs
        ),
        version,
    )


def read(
    href: str,
    *,
    format: str | None = None,
    options: list[tuple[str, str]] | None = None,
    root: Catalog | None = None,
) -> STACObject | ItemCollection:
    """
    Reads STAC from a href.

    Args:
        href (str): The href to write to
        format (str | None): The output format to write. If not provided, will be
            inferred from the href's extension.
        options (list[tuple[str, str]] | None): Options for configuring an
            object store, e.g. your AWS credentials.
        root (Catalog | None): Optional root of the catalog for this object. If
            provided, the root's resolved object cache can be used to search for
            previously resolved instances of the STAC object.

    Returns:
        STACObject | ItemCollection: The STAC value

    Examples:
        >>> item = stacrs.read("item.json")
    """
    d = stacrs.read(href, format=format, options=options)
    if d.get("type") == "FeatureCollection":
        return ItemCollection.from_dict(d)
    else:
        return pystac.read_dict(d, href=href, root=root)


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
) -> Iterable[Item]:
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

    Returns:
        Iterable[Item]: A iterable of pystac items.

    Examples:
        >>> items = list(stacrs.search(
        ...     "https://landsatlook.usgs.gov/stac-server",
        ...     collections=["landsat-c2l2-sr"],
        ...     intersects={"type": "Point", "coordinates": [-105.119, 40.173]},
        ...     sortby="-properties.datetime",
        ...     max_items=1,
        ... ))
    """
    items = stacrs.search(
        href,
        intersects=intersects,
        ids=ids,
        collections=collections,
        max_items=max_items,
        limit=limit,
        bbox=bbox,
        datetime=datetime,
        include=include,
        exclude=exclude,
        sortby=sortby,
        filter=filter,
        query=query,
    )
    return (Item.from_dict(d) for d in items)


def validate(
    value: STACObject,
    *,
    include_self_link: bool = True,
    transform_hrefs: bool = False,
) -> None:
    """
    Validates a STAC dictionary with json-schema.

    Args:
        value (STACObject): The STAC value to validate
        include_self_link (bool): Whether to include a self link when
            serializing this object to a dictionary.
        transform_hrefs (bool): Whether to transform this object's hrefs when
            serializing it to a dictionary. While pystac defaults to true, this
            function defaults to false
            (https://github.com/stac-utils/pystac/issues/960)

    Raises:
        Exception: On a validation error

    Examples:
        >>> item = pystac.read_file("examples/simple-item.json")
        >>> stacrs.pystac.validate(item)
    """
    return stacrs.validate(
        value.to_dict(
            include_self_link=include_self_link, transform_hrefs=transform_hrefs
        )
    )


def write(
    href: str,
    value: STACObject | list[Item],
    *,
    format: str | None = None,
    options: list[tuple[str, str]] | None = None,
    include_self_link: bool = True,
    transform_hrefs: bool = False,
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
        include_self_link (bool): Whether to include a self link when
            serializing this object to a dictionary.
        transform_hrefs (bool): Whether to transform this object's hrefs when
            serializing it to a dictionary. While pystac defaults to true, this
            function defaults to false
            (https://github.com/stac-utils/pystac/issues/960)

    Returns:
        dict[str, str] | None: The result of putting data into an object store,
            e.g. the e_tag and the version. None is returned if the file was written
            locally.

    Examples:
        >>> with open("simple-item.json") as f:
        ...     item = json.load(f)
        >>> stacrs.write("out.parquet", [item])
    """
    if isinstance(value, list):
        d = ItemCollection(value).to_dict(transform_hrefs=transform_hrefs)
    else:
        d = value.to_dict(
            include_self_link=include_self_link, transform_hrefs=transform_hrefs
        )
    return stacrs.write(str(href), d, format=format, options=options)
