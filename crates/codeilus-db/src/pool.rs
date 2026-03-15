//! Database connection management.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::Connection;
use std::path::Path;
use std::sync::{Arc, Mutex};

pub struct DbPool {
    conn: Arc<Mutex<Connection>>,
}

impl DbPool {
    pub fn new(path: &Path) -> CodeilusResult<Self> {
        let conn = Connection::open(path).map_err(|e| CodeilusError::Database(Box::new(e)))?;
        conn.execute_batch(
            "PRAGMA journal_mode = WAL;
             PRAGMA synchronous = NORMAL;
             PRAGMA foreign_keys = ON;
             PRAGMA busy_timeout = 5000;
             PRAGMA cache_size = -8000;",
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn in_memory() -> CodeilusResult<Self> {
        let conn =
            Connection::open_in_memory().map_err(|e| CodeilusError::Database(Box::new(e)))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(Self {
            conn: Arc::new(Mutex::new(conn)),
        })
    }

    pub fn connection(&self) -> std::sync::MutexGuard<'_, Connection> {
        self.conn.lock().expect("db mutex poisoned")
    }

    pub fn conn_arc(&self) -> Arc<Mutex<Connection>> {
        Arc::clone(&self.conn)
    }
}
