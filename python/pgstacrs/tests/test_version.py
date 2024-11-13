from pgstacrs import Client


async def test_version(client: Client) -> None:
    assert await client.get_version() is not None
