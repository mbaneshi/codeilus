//! Repository for user annotations on graph nodes and edges.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnnotationRow {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
    pub flagged: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub struct AnnotationRepo {
    conn: Arc<Mutex<Connection>>,
}

impl AnnotationRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(&self, target_type: &str, target_id: i64, content: &str) -> CodeilusResult<i64> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO annotations (target_type, target_id, content) VALUES (?1, ?2, ?3)",
            params![target_type, target_id, content],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn update(&self, id: i64, content: &str) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "UPDATE annotations SET content = ?1, updated_at = datetime('now') WHERE id = ?2",
            params![content, id],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn toggle_flag(&self, id: i64) -> CodeilusResult<bool> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "UPDATE annotations SET flagged = CASE WHEN flagged = 0 THEN 1 ELSE 0 END, updated_at = datetime('now') WHERE id = ?1",
            params![id],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let flagged: bool = conn
            .query_row(
                "SELECT flagged FROM annotations WHERE id = ?1",
                params![id],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(flagged)
    }

    pub fn delete(&self, id: i64) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM annotations WHERE id = ?1", params![id])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    pub fn list_by_target(
        &self,
        target_type: &str,
        target_id: i64,
    ) -> CodeilusResult<Vec<AnnotationRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, target_type, target_id, content, flagged, created_at, updated_at \
                 FROM annotations WHERE target_type = ?1 AND target_id = ?2 ORDER BY created_at DESC",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![target_type, target_id], |row| {
                Ok(AnnotationRow {
                    id: row.get(0)?,
                    target_type: row.get(1)?,
                    target_id: row.get(2)?,
                    content: row.get(3)?,
                    flagged: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn list_all(&self) -> CodeilusResult<Vec<AnnotationRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, target_type, target_id, content, flagged, created_at, updated_at \
                 FROM annotations ORDER BY flagged DESC, created_at DESC",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(AnnotationRow {
                    id: row.get(0)?,
                    target_type: row.get(1)?,
                    target_id: row.get(2)?,
                    content: row.get(3)?,
                    flagged: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn list_flagged(&self) -> CodeilusResult<Vec<AnnotationRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, target_type, target_id, content, flagged, created_at, updated_at \
                 FROM annotations WHERE flagged = 1 ORDER BY created_at DESC",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(AnnotationRow {
                    id: row.get(0)?,
                    target_type: row.get(1)?,
                    target_id: row.get(2)?,
                    content: row.get(3)?,
                    flagged: row.get(4)?,
                    created_at: row.get(5)?,
                    updated_at: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn delete_all(&self) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM annotations", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
