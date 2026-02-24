use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::{QueryFragment, QueryId};
use diesel::sqlite::{Sqlite, SqliteConnection};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnection, RunQueryDsl, pg::AsyncPgConnection};
use std::env;
use tracing::{info, warn};

pub struct Orchestrator {
    pub sqlite: SyncConnectionWrapper<SqliteConnection>,
    pub pg: Option<AsyncPgConnection>,
}

const SCHEMA_SQL: &str = "
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS users (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    username TEXT NOT NULL UNIQUE,
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    passkey TEXT,
    eakey TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS user_property (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    instance_status TEXT NOT NULL DEFAULT 'inactive',
    instance_usage REAL NOT NULL DEFAULT 0.0,
    api_key_active INTEGER NOT NULL DEFAULT 0,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS instances (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    instances_count INTEGER NOT NULL DEFAULT 0,
    expected_consumption REAL NOT NULL DEFAULT 0.0,
    instances_overall_consumption REAL NOT NULL DEFAULT 0.0,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

CREATE TABLE IF NOT EXISTS billing (
    id INTEGER NOT NULL PRIMARY KEY AUTOINCREMENT,
    user_id INTEGER NOT NULL,
    amount_in_wallet REAL NOT NULL DEFAULT 0.0,
    amount_spent REAL NOT NULL DEFAULT 0.0,
    total_amount_spent REAL NOT NULL DEFAULT 0.0,
    average_hourly_consumption REAL NOT NULL DEFAULT 0.0,
    FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
);

ALTER TABLE user_property ADD COLUMN IF NOT EXISTS api_key_active INTEGER NOT NULL DEFAULT 0;
";

impl Orchestrator {
    pub async fn init() -> Self {
        let sqlite_url = env::var("SQLITE_DATABASE_URL").unwrap_or_else(|_| "database.db".to_string());
        let pg_url = env::var("POSTGRES_DATABASE_URL").ok().filter(|s| !s.is_empty());

        info!("Connecting to SQLite at {}", sqlite_url);

        let mut sqlite_conn = SyncConnectionWrapper::<SqliteConnection>::establish(&sqlite_url)
            .await
            .expect("SQLite must start");

        // Apply schema for each statement individually
        for stmt in SCHEMA_SQL.split(';') {
            let trimmed = stmt.trim();
            if trimmed.is_empty() {
                continue;
            }
            let query = diesel::sql_query(trimmed);
            let _ = query.execute(&mut sqlite_conn).await.map_err(|e| {
                warn!("Schema init error: {}", e);
            });
        }

        let pg_conn = if let Some(url) = pg_url {
            info!("Connecting to Postgres...");
            match AsyncPgConnection::establish(&url).await {
                Ok(c) => {
                    info!("Postgres connected.");
                    Some(c)
                }
                Err(e) => {
                    warn!("Postgres offline: {}. Operating in SQLite-only mode.", e);
                    None
                }
            }
        } else {
            info!("No POSTGRES_DATABASE_URL set — SQLite-only mode.");
            None
        };

        Self {
            sqlite: sqlite_conn,
            pg: pg_conn,
        }
    }

    pub async fn sync_write<T>(&mut self, query: T) -> QueryResult<usize>
    where
        T: RunQueryDsl<SyncConnectionWrapper<SqliteConnection>>
            + RunQueryDsl<AsyncPgConnection>
            + QueryFragment<Sqlite>
            + QueryFragment<Pg>
            + QueryId
            + Send
            + Clone
            + 'static,
    {
        let result = query.clone().execute(&mut self.sqlite).await;

        if let Some(ref mut pg_conn) = self.pg {
            let _ = query.execute(pg_conn).await.map_err(|e| {
                eprintln!("Postgres Sync Failed: {}", e);
            });
        }

        result
    }
}
