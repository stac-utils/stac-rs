from typing import Any

class Client:
    """A PgSTAC client."""

    @classmethod
    async def open(cls, dsn: str) -> Client:
        """Opens a new client with the provided connection information.

        The connection string can be:

            postgresql://username:password@hostname:port/dbname

        Or:

            host=hostname port=port user=username password=password dbname=dbname
        """

    async def get_version(self) -> str:
        """Returns the PgSTAC version."""

    async def get_collection(self, id: str) -> dict[str, Any] | None:
        """Returns the collection with the provided id, or None if the collection does not exist"""

    async def create_collection(self, collection: dict[str, Any]) -> None:
        """Creates a collection."""

    async def _execute_query(self, query: str) -> None:
        """Executes a single query without any parameters."""
