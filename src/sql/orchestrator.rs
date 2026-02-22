use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::query_builder::{QueryFragment, QueryId};
use diesel::sqlite::{Sqlite, SqliteConnection};
use diesel_async::sync_connection_wrapper::SyncConnectionWrapper;
use diesel_async::{AsyncConnection, RunQueryDsl, pg::AsyncPgConnection};
use std::env;

pub struct Orchestrator {
    sqlite: SyncConnectionWrapper<SqliteConnection>,
    pg: Option<AsyncPgConnection>,
}

impl Orchestrator {
    pub async fn init() -> Self {
        let db_url = env::var("DATABASE_URL").unwrap_or_else(|_| "database.db".to_string());

        let sqlite_conn = SyncConnectionWrapper::<SqliteConnection>::establish("database.db")
            .await
            .expect("SQLite must start");

        let pg_conn = if db_url.starts_with("postgres") {
            match AsyncPgConnection::establish(&db_url).await {
                Ok(c) => Some(c),
                Err(e) => {
                    eprintln!("Postgres offline: {}. Operating in SQLite-only mode.", e);
                    None
                }
            }
        } else {
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
