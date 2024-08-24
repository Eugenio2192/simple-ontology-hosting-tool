rm sqlite.db
rm sqlite.db-shm
rm sqlite.db-wal
DATABASE_URL=sqlite://sqlite.db sqlx db create
DATABASE_URL=sqlite://sqlite.db sqlx migrate run
DATABASE_URL=sqlite://sqlite.db cargo run
