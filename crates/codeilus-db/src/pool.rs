//! Database connection pool using r2d2 + SQLite.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use r2d2::Pool;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;
use std::path::Path;
use std::sync::atomic::{AtomicU64, Ordering};

pub type PooledConn = r2d2::PooledConnection<SqliteConnectionManager>;

static IN_MEMORY_COUNTER: AtomicU64 = AtomicU64::new(0);

pub struct DbPool {
    pool: Pool<SqliteConnectionManager>,
}

impl DbPool {
    pub fn new(path: &Path) -> CodeilusResult<Self> {
        let manager = SqliteConnectionManager::file(path).with_init(|conn| {
            conn.execute_batch(
                "PRAGMA journal_mode = WAL;
                 PRAGMA synchronous = NORMAL;
                 PRAGMA foreign_keys = ON;
                 PRAGMA busy_timeout = 5000;
                 PRAGMA cache_size = -8000;",
            )
        });
        let pool = Pool::builder()
            .max_size(8)
            .build(manager)
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(Self { pool })
    }

    pub fn in_memory() -> CodeilusResult<Self> {
        let id = IN_MEMORY_COUNTER.fetch_add(1, Ordering::Relaxed);
        let uri = format!("file:codeilus_mem_{id}?mode=memory&cache=shared");
        let manager = SqliteConnectionManager::file(uri)
            .with_flags(
                OpenFlags::SQLITE_OPEN_URI
                    | OpenFlags::SQLITE_OPEN_READ_WRITE
                    | OpenFlags::SQLITE_OPEN_CREATE,
            )
            .with_init(|conn| conn.execute_batch("PRAGMA foreign_keys = ON;"));
        let pool = Pool::builder()
            .max_size(4)
            .build(manager)
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(Self { pool })
    }

    /// Get a pooled connection.
    pub fn connection(&self) -> PooledConn {
        self.pool.get().expect("db pool exhausted")
    }
}
