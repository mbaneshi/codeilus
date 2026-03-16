use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::pool::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRow {
    pub id: i64,
    pub kind: String,
    pub severity: String,
    pub file_id: Option<i64>,
    pub symbol_id: Option<i64>,
    pub description: String,
}

pub struct PatternRepo {
    db: Arc<DbPool>,
}

impl PatternRepo {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// Insert a single pattern finding. Returns the new row ID.
    pub fn insert(&self, row: &PatternRow) -> CodeilusResult<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO patterns (kind, severity, file_id, symbol_id, description) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![row.kind, row.severity, row.file_id, row.symbol_id, row.description],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(conn.last_insert_rowid())
    }

    /// Batch insert pattern findings in a transaction.
    pub fn insert_batch(&self, findings: &[PatternRow]) -> CodeilusResult<Vec<i64>> {
        let conn = self.db.connection();
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(findings.len());
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO patterns (kind, severity, file_id, symbol_id, description) VALUES (?1, ?2, ?3, ?4, ?5)",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for row in findings {
                stmt.execute(params![
                    row.kind,
                    row.severity,
                    row.file_id,
                    row.symbol_id,
                    row.description,
                ])
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                ids.push(tx.last_insert_rowid());
            }
        }
        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(ids)
    }

    /// List all pattern findings.
    pub fn list(&self) -> CodeilusResult<Vec<PatternRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("SELECT id, kind, severity, file_id, symbol_id, description FROM patterns")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(PatternRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    severity: row.get(2)?,
                    file_id: row.get(3)?,
                    symbol_id: row.get(4)?,
                    description: row.get(5)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// List findings filtered by severity.
    pub fn list_by_severity(&self, severity: &str) -> CodeilusResult<Vec<PatternRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, severity, file_id, symbol_id, description FROM patterns WHERE severity = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![severity], |row| {
                Ok(PatternRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    severity: row.get(2)?,
                    file_id: row.get(3)?,
                    symbol_id: row.get(4)?,
                    description: row.get(5)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// List findings filtered by kind.
    pub fn list_by_kind(&self, kind: &str) -> CodeilusResult<Vec<PatternRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, severity, file_id, symbol_id, description FROM patterns WHERE kind = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![kind], |row| {
                Ok(PatternRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    severity: row.get(2)?,
                    file_id: row.get(3)?,
                    symbol_id: row.get(4)?,
                    description: row.get(5)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// List findings filtered by file_id.
    pub fn list_by_file(&self, file_id: i64) -> CodeilusResult<Vec<PatternRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, kind, severity, file_id, symbol_id, description FROM patterns WHERE file_id = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![file_id], |row| {
                Ok(PatternRow {
                    id: row.get(0)?,
                    kind: row.get(1)?,
                    severity: row.get(2)?,
                    file_id: row.get(3)?,
                    symbol_id: row.get(4)?,
                    description: row.get(5)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Count findings grouped by severity.
    pub fn count_by_severity(&self) -> CodeilusResult<Vec<(String, usize)>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("SELECT severity, COUNT(*) FROM patterns GROUP BY severity")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)? as usize))
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Delete all pattern findings.
    pub fn delete_all(&self) -> CodeilusResult<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM patterns", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
