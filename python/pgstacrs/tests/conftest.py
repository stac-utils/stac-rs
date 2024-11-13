import json
from pathlib import Path
from typing import Any, AsyncGenerator, Generator

import pytest
import pytest_postgresql.factories
from pgstacrs import Client
from pytest_postgresql.executor import PostgreSQLExecutor
from pytest_postgresql.janitor import DatabaseJanitor

pgstac_template = pytest_postgresql.factories.postgresql_proc(
    load=["tests.migrate.pgstac"]
)


@pytest.fixture(scope="session")
def pgstac_dsn(
    pgstac_template: PostgreSQLExecutor,
) -> Generator[str, None, None]:
    with DatabaseJanitor(
        user=pgstac_template.user,
        host=pgstac_template.host,
        port=pgstac_template.port,
        dbname="pgstac_test_database",
        version=pgstac_template.version,
        password="rugreallytiedtheroomtogether",
        template_dbname=pgstac_template.dbname + "_tmpl",
    ):
        yield f"postgresql://{pgstac_template.user}:rugreallytiedtheroomtogether@{pgstac_template.host}:{pgstac_template.port}/pgstac_test_database"


@pytest.fixture
async def client(pgstac_dsn: str) -> AsyncGenerator[Client, None]:
    client = await Client.open(pgstac_dsn)
    client._execute_query("BEGIN")
    try:
        yield client
    finally:
        client._execute_query("ROLLBACK")


@pytest.fixture
def collection() -> dict[str, Any]:
    with open(Path(__file__).parents[1] / "examples" / "collection.json") as f:
        return json.load(f)
