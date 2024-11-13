from pypgstac.db import PgstacDB
from pypgstac.migrate import Migrate


def pgstac(host, port, user, dbname, password) -> None:
    db = PgstacDB(f"postgresql://{user}:{password}@{host}:{port}/{dbname}")
    Migrate(db).run_migration()
