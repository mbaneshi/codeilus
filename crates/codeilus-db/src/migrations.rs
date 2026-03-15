//! Schema migration runner.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::Connection;
use tracing::info;

const MIGRATION_001: &str = include_str!("../../../migrations/0001_init.sql");
const MIGRATION_002: &str = include_str!("../../../migrations/0002_fts5.sql");
const MIGRATION_003: &str = include_str!("../../../migrations/0003_quiz_columns.sql");

pub struct Migrator<'a> {
    conn: &'a Connection,
}

impl<'a> Migrator<'a> {
    pub fn new(conn: &'a Connection) -> Self {
        Self { conn }
    }

    pub fn current_version(&self) -> CodeilusResult<u32> {
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='schema_version'",
                [],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        if !exists {
            return Ok(0);
        }

        let version: u32 = self
            .conn
            .query_row(
                "SELECT COALESCE(MAX(version), 0) FROM schema_version",
                [],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(version)
    }

    pub fn apply_pending(&self) -> CodeilusResult<u32> {
        let current = self.current_version()?;
        let mut applied = 0;

        if current < 1 {
            info!("applying migration 0001_init.sql");
            self.conn
                .execute_batch(MIGRATION_001)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0001 applied, now at version 1");
        }

        if current < 2 {
            info!("applying migration 0002_fts5.sql");
            self.conn
                .execute_batch(MIGRATION_002)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0002 applied, now at version 2");
        }

        if current < 3 {
            info!("applying migration 0003_quiz_columns.sql");
            self.conn
                .execute_batch(MIGRATION_003)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            self.conn
                .execute("INSERT INTO schema_version (version) VALUES (3)", [])
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0003 applied, now at version 3");
        }

        if applied == 0 {
            info!(version = current, "schema already at latest version");
        }
        Ok(applied)
    }
}
