//! Schema migration runner.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::Connection;
use tracing::info;

const MIGRATION_001: &str = include_str!("../../../migrations/0001_init.sql");
const MIGRATION_002: &str = include_str!("../../../migrations/0002_fts5.sql");
const MIGRATION_003: &str = include_str!("../../../migrations/0003_quiz_columns.sql");
const MIGRATION_004: &str = include_str!("../../../migrations/0004_annotations.sql");
const MIGRATION_005: &str = include_str!("../../../migrations/0005_progress_unique.sql");
const MIGRATION_006: &str = include_str!("../../../migrations/0006_seed_badges.sql");
const MIGRATION_007: &str = include_str!("../../../migrations/0007_narrative_placeholder.sql");
const MIGRATION_008: &str = include_str!("../../../migrations/0008_content_hash.sql");
const MIGRATION_009: &str = include_str!("../../../migrations/0009_pipeline_runs.sql");
const MIGRATION_010: &str = include_str!("../../../migrations/0010_add_indexes.sql");

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

        if current < 4 {
            info!("applying migration 0004_annotations.sql");
            self.conn
                .execute_batch(MIGRATION_004)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            self.conn
                .execute(
                    "INSERT INTO schema_version (version) VALUES (4)",
                    [],
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0004 applied, now at version 4");
        }

        if current < 5 {
            info!("applying migration 0005_progress_unique.sql");
            self.conn
                .execute_batch(MIGRATION_005)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            self.conn
                .execute(
                    "INSERT INTO schema_version (version) VALUES (5)",
                    [],
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0005 applied, now at version 5");
        }

        if current < 6 {
            info!("applying migration 0006_seed_badges.sql");
            self.conn
                .execute_batch(MIGRATION_006)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            self.conn
                .execute(
                    "INSERT INTO schema_version (version) VALUES (6)",
                    [],
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0006 applied, now at version 6");
        }

        if current < 7 {
            info!("applying migration 0007_narrative_placeholder.sql");
            self.conn
                .execute_batch(MIGRATION_007)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            self.conn
                .execute(
                    "INSERT INTO schema_version (version) VALUES (7)",
                    [],
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0007 applied, now at version 7");
        }

        if current < 8 {
            info!("applying migration 0008_content_hash.sql");
            self.conn
                .execute_batch(MIGRATION_008)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            self.conn
                .execute(
                    "INSERT INTO schema_version (version) VALUES (8)",
                    [],
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0008 applied, now at version 8");
        }

        if current < 9 {
            info!("applying migration 0009_pipeline_runs.sql");
            self.conn
                .execute_batch(MIGRATION_009)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            self.conn
                .execute(
                    "INSERT INTO schema_version (version) VALUES (9)",
                    [],
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0009 applied, now at version 9");
        }

        if current < 10 {
            info!("applying migration 0010_add_indexes.sql");
            self.conn
                .execute_batch(MIGRATION_010)
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            applied += 1;
            info!("migration 0010 applied, now at version 10");
        }

        if applied == 0 {
            info!(version = current, "schema already at latest version");
        }
        Ok(applied)
    }
}
