use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::ids::{EdgeId, SymbolId};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRow {
    pub id: EdgeId,
    pub source_id: SymbolId,
    pub target_id: SymbolId,
    pub kind: String,
    pub confidence: f64,
}

pub struct EdgeRepo {
    conn: Arc<Mutex<Connection>>,
}

impl EdgeRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Insert a single edge.
    pub fn insert(
        &self,
        source_id: SymbolId,
        target_id: SymbolId,
        kind: &str,
        confidence: f64,
    ) -> CodeilusResult<EdgeId> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO edges (source_id, target_id, kind, confidence) VALUES (?1, ?2, ?3, ?4)",
            params![source_id.0, target_id.0, kind, confidence],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(EdgeId(conn.last_insert_rowid()))
    }

    /// Batch insert edges in a transaction.
    pub fn insert_batch(
        &self,
        edges: &[(SymbolId, SymbolId, String, f64)],
    ) -> CodeilusResult<Vec<EdgeId>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(edges.len());
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO edges (source_id, target_id, kind, confidence) VALUES (?1, ?2, ?3, ?4)",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for (source_id, target_id, kind, confidence) in edges {
                stmt.execute(params![source_id.0, target_id.0, kind, confidence])
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                ids.push(EdgeId(tx.last_insert_rowid()));
            }
        }
        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(ids)
    }

    /// List edges from a symbol (outgoing).
    pub fn list_from(&self, source_id: SymbolId) -> CodeilusResult<Vec<EdgeRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, source_id, target_id, kind, confidence FROM edges WHERE source_id = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![source_id.0], |row| {
                Ok(EdgeRow {
                    id: EdgeId(row.get(0)?),
                    source_id: SymbolId(row.get(1)?),
                    target_id: SymbolId(row.get(2)?),
                    kind: row.get(3)?,
                    confidence: row.get(4)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// List edges to a symbol (incoming).
    pub fn list_to(&self, target_id: SymbolId) -> CodeilusResult<Vec<EdgeRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, source_id, target_id, kind, confidence FROM edges WHERE target_id = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![target_id.0], |row| {
                Ok(EdgeRow {
                    id: EdgeId(row.get(0)?),
                    source_id: SymbolId(row.get(1)?),
                    target_id: SymbolId(row.get(2)?),
                    kind: row.get(3)?,
                    confidence: row.get(4)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// List all edges of a specific kind.
    pub fn list_by_kind(&self, kind: &str) -> CodeilusResult<Vec<EdgeRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, source_id, target_id, kind, confidence FROM edges WHERE kind = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![kind], |row| {
                Ok(EdgeRow {
                    id: EdgeId(row.get(0)?),
                    source_id: SymbolId(row.get(1)?),
                    target_id: SymbolId(row.get(2)?),
                    kind: row.get(3)?,
                    confidence: row.get(4)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Count total edges.
    pub fn count(&self) -> CodeilusResult<usize> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(count as usize)
    }

    /// Delete all edges (for re-analysis).
    pub fn delete_all(&self) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM edges", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
