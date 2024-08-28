class StacrsError(Exception):
    """Custom exception type for errors originating in the stacrs package."""

def validate_href(href: str) -> None:
    """Validates a STAC value at the provided href, raising an exception on any validation errors."""
