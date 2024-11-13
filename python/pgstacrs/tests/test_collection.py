from typing import Any

from pgstacrs import Client


async def test_create_collection(client: Client, collection: dict[str, Any]) -> None:
    await client.create_collection(collection)
    assert await client.get_collection("simple-collection") is not None


async def test_get_collection(client: Client) -> None:
    assert await client.get_collection("does-not-exist") is None
