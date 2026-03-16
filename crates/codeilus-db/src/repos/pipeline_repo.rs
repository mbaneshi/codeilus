use crate::pool::DbPool;
use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::params;
use std::sync::Arc;

pub struct PipelineRepo {
    db: Arc<DbPool>,
}

impl PipelineRepo {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    pub fn mark_started(&self, repo_path: &str, step: &str) -> CodeilusResult<()> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO pipeline_runs (repo_path, step_name, status, started_at)
             VALUES (?1, ?2, 'running', datetime('now'))
             ON CONFLICT(repo_path, step_name) DO UPDATE SET status = 'running', started_at = datetime('now'), error_message = NULL",
            params![repo_path, step],
        ).map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn mark_completed(&self, repo_path: &str, step: &str) -> CodeilusResult<()> {
        let conn = self.db.connection();
        conn.execute(
            "UPDATE pipeline_runs SET status = 'completed', completed_at = datetime('now') WHERE repo_path = ?1 AND step_name = ?2",
            params![repo_path, step],
        ).map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn mark_failed(&self, repo_path: &str, step: &str, error: &str) -> CodeilusResult<()> {
        let conn = self.db.connection();
        conn.execute(
            "UPDATE pipeline_runs SET status = 'failed', error_message = ?3 WHERE repo_path = ?1 AND step_name = ?2",
            params![repo_path, step, error],
        ).map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn is_completed(&self, repo_path: &str, step: &str) -> bool {
        let conn = self.db.connection();
        conn.query_row(
            "SELECT COUNT(*) FROM pipeline_runs WHERE repo_path = ?1 AND step_name = ?2 AND status = 'completed'",
            params![repo_path, step],
            |row| row.get::<_, i64>(0),
        ).unwrap_or(0) > 0
    }

    pub fn reset(&self, repo_path: &str) -> CodeilusResult<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM pipeline_runs WHERE repo_path = ?1", params![repo_path])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
