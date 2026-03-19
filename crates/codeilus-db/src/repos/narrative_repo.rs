use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::pool::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeRow {
    pub id: i64,
    pub kind: String,
    pub target_id: Option<i64>,
    pub language: String,
    pub content: String,
    pub generated_at: String,
    pub is_placeholder: bool,
}

pub struct NarrativeRepo {
    db: Arc<DbPool>,
}

impl NarrativeRepo {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// Insert a single narrative.
    pub fn insert(
        &self,
        kind: &str,
        target_id: Option<i64>,
        content: &str,
        is_placeholder: bool,
    ) -> CodeilusResult<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO narratives (kind, target_id, content, is_placeholder) VALUES (?1, ?2, ?3, ?4)",
            params![kind, target_id, content, is_placeholder],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(conn.last_insert_rowid())
    }

    /// Batch insert narratives in a transaction.
    pub fn insert_batch(
        &self,
        narratives: &[(String, Option<i64>, String, bool)],
    ) -> CodeilusResult<Vec<i64>> {
        let conn = self.db.connection();
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(narratives.len());
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO narratives (kind, target_id, content, is_placeholder) VALUES (?1, ?2, ?3, ?4)",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for (kind, target_id, content, is_placeholder) in narratives {
                stmt.execute(params![kind, target_id, content, is_placeholder])
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                ids.push(tx.last_insert_rowid());
            }
        }
        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(ids)
    }

    /// Get first narrative matching a kind.
    pub fn get_by_kind(&self, kind: &str) -> CodeilusResult<Option<NarrativeRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, target_id, language, content, generated_at, is_placeholder FROM narratives WHERE kind = ?1 LIMIT 1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut rows = stmt
            .query_map(params![kind], |row| {
                Ok(NarrativeRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    target_id: row.get(2)?,
                    language: row.get(3)?,
                    content: row.get(4)?,
                    generated_at: row.get(5)?,
                    is_placeholder: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        match rows.next() {
            Some(row) => Ok(Some(
                row.map_err(|e| CodeilusError::Database(Box::new(e)))?,
            )),
            None => Ok(None),
        }
    }

    /// Get narrative by kind and target_id.
    pub fn get_by_kind_and_target(
        &self,
        kind: &str,
        target_id: i64,
    ) -> CodeilusResult<Option<NarrativeRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, target_id, language, content, generated_at, is_placeholder FROM narratives WHERE kind = ?1 AND target_id = ?2 LIMIT 1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut rows = stmt
            .query_map(params![kind, target_id], |row| {
                Ok(NarrativeRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    target_id: row.get(2)?,
                    language: row.get(3)?,
                    content: row.get(4)?,
                    generated_at: row.get(5)?,
                    is_placeholder: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        match rows.next() {
            Some(row) => Ok(Some(
                row.map_err(|e| CodeilusError::Database(Box::new(e)))?,
            )),
            None => Ok(None),
        }
    }

    /// List all narratives.
    pub fn list(&self) -> CodeilusResult<Vec<NarrativeRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, target_id, language, content, generated_at, is_placeholder FROM narratives",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(NarrativeRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    target_id: row.get(2)?,
                    language: row.get(3)?,
                    content: row.get(4)?,
                    generated_at: row.get(5)?,
                    is_placeholder: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// List narratives with pagination.
    pub fn list_paginated(&self, limit: i64, offset: i64) -> CodeilusResult<Vec<NarrativeRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, target_id, language, content, generated_at, is_placeholder FROM narratives LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![limit, offset], |row| {
                Ok(NarrativeRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    target_id: row.get(2)?,
                    language: row.get(3)?,
                    content: row.get(4)?,
                    generated_at: row.get(5)?,
                    is_placeholder: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// List narratives by kind.
    pub fn list_by_kind(&self, kind: &str) -> CodeilusResult<Vec<NarrativeRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, target_id, language, content, generated_at, is_placeholder FROM narratives WHERE kind = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![kind], |row| {
                Ok(NarrativeRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    target_id: row.get(2)?,
                    language: row.get(3)?,
                    content: row.get(4)?,
                    generated_at: row.get(5)?,
                    is_placeholder: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Delete all narratives.
    pub fn delete_all(&self) -> CodeilusResult<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM narratives", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
