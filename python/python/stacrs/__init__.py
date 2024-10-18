import pystac
from .pystac import (
    migrate,
    read,
    search,
    validate,
    write,
)

from .stacrs import (
    migrate_href,
    search_to,
    validate_href,
)

__doc__ = pystac.__doc__
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
