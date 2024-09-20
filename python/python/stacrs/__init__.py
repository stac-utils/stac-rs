from . import stacrs
from .stacrs import (
    migrate,
    migrate_href,
    read,
    search,
    search_to,
    validate,
    validate_href,
    write,
)

__doc__ = stacrs.__doc__
__all__ = [
    "migrate",
    "migrate_href",
    "read",
    "search",
    "search_to",
    "validate",
    "validate_href",
    "write",
]

import importlib.util

if importlib.util.find_spec("pystac") is not None:
    from . import pystac as pystac

    __all__.append("pystac")
