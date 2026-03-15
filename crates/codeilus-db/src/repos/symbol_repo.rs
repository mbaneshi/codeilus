use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::ids::{FileId, SymbolId};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRow {
    pub id: SymbolId,
    pub file_id: FileId,
    pub name: String,
    pub kind: String,
    pub start_line: i64,
    pub end_line: i64,
    pub signature: Option<String>,
}

/// Input for batch symbol insertion.
pub type NewSymbolBatch = (FileId, String, String, i64, i64, Option<String>);

pub struct SymbolRepo {
    conn: Arc<Mutex<Connection>>,
}

impl SymbolRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Insert a single symbol.
    pub fn insert(
        &self,
        file_id: FileId,
        name: &str,
        kind: &str,
        start_line: i64,
        end_line: i64,
        signature: Option<&str>,
    ) -> CodeilusResult<SymbolId> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO symbols (file_id, name, kind, start_line, end_line, signature) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![file_id.0, name, kind, start_line, end_line, signature],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(SymbolId(conn.last_insert_rowid()))
    }

    /// Batch insert symbols in a transaction.
    pub fn insert_batch(
        &self,
        symbols: &[NewSymbolBatch],
    ) -> CodeilusResult<Vec<SymbolId>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(symbols.len());
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO symbols (file_id, name, kind, start_line, end_line, signature) VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for (file_id, name, kind, start_line, end_line, signature) in symbols {
                stmt.execute(params![file_id.0, name, kind, start_line, end_line, signature])
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                ids.push(SymbolId(tx.last_insert_rowid()));
            }
        }
        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(ids)
    }

    /// Get symbol by ID.
    pub fn get(&self, id: SymbolId) -> CodeilusResult<SymbolRow> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.query_row(
            "SELECT id, file_id, name, kind, start_line, end_line, signature FROM symbols WHERE id = ?1",
            params![id.0],
            |row| {
                Ok(SymbolRow {
                    id: SymbolId(row.get(0)?),
                    file_id: FileId(row.get(1)?),
                    name: row.get(2)?,
                    kind: row.get(3)?,
                    start_line: row.get(4)?,
                    end_line: row.get(5)?,
                    signature: row.get(6)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CodeilusError::NotFound(format!("symbol with id {id}"))
            }
            other => CodeilusError::Database(Box::new(other)),
        })
    }

    /// List symbols for a file.
    pub fn list_by_file(&self, file_id: FileId) -> CodeilusResult<Vec<SymbolRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, file_id, name, kind, start_line, end_line, signature FROM symbols WHERE file_id = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![file_id.0], |row| {
                Ok(SymbolRow {
                    id: SymbolId(row.get(0)?),
                    file_id: FileId(row.get(1)?),
                    name: row.get(2)?,
                    kind: row.get(3)?,
                    start_line: row.get(4)?,
                    end_line: row.get(5)?,
                    signature: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Search symbols by name (exact match).
    pub fn list_by_name(&self, name: &str) -> CodeilusResult<Vec<SymbolRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT id, file_id, name, kind, start_line, end_line, signature FROM symbols WHERE name = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![name], |row| {
                Ok(SymbolRow {
                    id: SymbolId(row.get(0)?),
                    file_id: FileId(row.get(1)?),
                    name: row.get(2)?,
                    kind: row.get(3)?,
                    start_line: row.get(4)?,
                    end_line: row.get(5)?,
                    signature: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Search symbols by name prefix (LIKE 'name%').
    pub fn search(&self, query: &str) -> CodeilusResult<Vec<SymbolRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let pattern = format!("{query}%");
        let mut stmt = conn
            .prepare(
                "SELECT id, file_id, name, kind, start_line, end_line, signature FROM symbols WHERE name LIKE ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![pattern], |row| {
                Ok(SymbolRow {
                    id: SymbolId(row.get(0)?),
                    file_id: FileId(row.get(1)?),
                    name: row.get(2)?,
                    kind: row.get(3)?,
                    start_line: row.get(4)?,
                    end_line: row.get(5)?,
                    signature: row.get(6)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Count total symbols.
    pub fn count(&self) -> CodeilusResult<usize> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(count as usize)
    }

    /// Delete all symbols for a file (for re-parse).
    pub fn delete_by_file(&self, file_id: FileId) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM symbols WHERE file_id = ?1", params![file_id.0])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    /// Delete all symbols (for re-analysis).
    pub fn delete_all(&self) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM symbols", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
