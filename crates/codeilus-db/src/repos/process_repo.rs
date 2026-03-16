use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::ids::SymbolId;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::pool::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessRow {
    pub id: i64,
    pub name: String,
    pub entry_symbol_id: SymbolId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStepRow {
    pub id: i64,
    pub process_id: i64,
    pub step_order: i64,
    pub symbol_id: SymbolId,
    pub description: String,
}

pub struct ProcessRepo {
    db: Arc<DbPool>,
}

impl ProcessRepo {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    pub fn insert(&self, name: &str, entry_symbol_id: SymbolId) -> CodeilusResult<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO processes (name, entry_symbol_id, description) VALUES (?1, ?2, '')",
            params![name, entry_symbol_id.0],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn insert_step(
        &self,
        process_id: i64,
        step_order: i64,
        symbol_id: SymbolId,
        description: &str,
    ) -> CodeilusResult<i64> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO process_steps (process_id, step_order, symbol_id) VALUES (?1, ?2, ?3)",
            params![process_id, step_order, symbol_id.0],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let _ = description; // process_steps table doesn't have description column
        Ok(conn.last_insert_rowid())
    }

    pub fn get(&self, id: i64) -> CodeilusResult<ProcessRow> {
        let conn = self.db.connection();
        conn.query_row(
            "SELECT id, name, entry_symbol_id FROM processes WHERE id = ?1",
            params![id],
            |row| {
                Ok(ProcessRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    entry_symbol_id: SymbolId(row.get(2)?),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CodeilusError::NotFound(format!("Process {id} not found"))
            }
            _ => CodeilusError::Database(Box::new(e)),
        })
    }

    pub fn list(&self) -> CodeilusResult<Vec<ProcessRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("SELECT id, name, entry_symbol_id FROM processes")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ProcessRow {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    entry_symbol_id: SymbolId(row.get(2)?),
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn list_steps(&self, process_id: i64) -> CodeilusResult<Vec<ProcessStepRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT rowid, process_id, step_order, symbol_id FROM process_steps WHERE process_id = ?1 ORDER BY step_order",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![process_id], |row| {
                Ok(ProcessStepRow {
                    id: row.get(0)?,
                    process_id: row.get(1)?,
                    step_order: row.get(2)?,
                    symbol_id: SymbolId(row.get(3)?),
                    description: String::new(),
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
        let conn = self.db.connection();
        conn.execute("DELETE FROM process_steps", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        conn.execute("DELETE FROM processes", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
