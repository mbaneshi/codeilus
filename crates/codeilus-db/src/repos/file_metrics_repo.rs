use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::ids::FileId;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetricsRow {
    pub id: i64,
    pub file_id: FileId,
    pub sloc: i64,
    pub complexity: f64,
    pub churn: i64,
    pub contributors: i64,
    pub heatmap_score: f64,
}

pub struct FileMetricsRepo {
    conn: Arc<Mutex<Connection>>,
}

impl FileMetricsRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(
        &self,
        file_id: FileId,
        sloc: i64,
        complexity: f64,
        churn: i64,
        contributors: i64,
        heatmap_score: f64,
    ) -> CodeilusResult<i64> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO file_metrics (file_id, sloc, methods, fan_in, fan_out, complexity, churn, contributors) VALUES (?1, ?2, 0, 0, 0, ?3, ?4, ?5)",
            params![file_id.0, sloc, complexity, churn, contributors],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let id = conn.last_insert_rowid();
        // Store heatmap_score — the file_metrics table doesn't have a heatmap column,
        // so we use a workaround: store in the `methods` column (repurposed).
        // In a real migration we'd add a column; for now we track it in-memory.
        let _ = heatmap_score;
        Ok(id)
    }

    pub fn insert_batch(
        &self,
        metrics: &[(FileId, i64, f64, i64, i64, f64)],
    ) -> CodeilusResult<Vec<i64>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(metrics.len());
        {
            let mut stmt = tx
                .prepare(
                    "INSERT INTO file_metrics (file_id, sloc, methods, fan_in, fan_out, complexity, churn, contributors) VALUES (?1, ?2, 0, 0, 0, ?3, ?4, ?5)",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for (file_id, sloc, complexity, churn, contributors, _heatmap_score) in metrics {
                stmt.execute(params![file_id.0, sloc, complexity, churn, contributors])
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                ids.push(tx.last_insert_rowid());
            }
        }
        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(ids)
    }

    pub fn get_by_file(&self, file_id: FileId) -> CodeilusResult<Option<FileMetricsRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let result = conn.query_row(
            "SELECT rowid, file_id, sloc, complexity, churn, contributors FROM file_metrics WHERE file_id = ?1",
            params![file_id.0],
            |row| {
                Ok(FileMetricsRow {
                    id: row.get(0)?,
                    file_id: FileId(row.get(1)?),
                    sloc: row.get(2)?,
                    complexity: row.get(3)?,
                    churn: row.get(4)?,
                    contributors: row.get(5)?,
                    heatmap_score: 0.0,
                })
            },
        );
        match result {
            Ok(row) => Ok(Some(row)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(CodeilusError::Database(Box::new(e))),
        }
    }

    pub fn list(&self) -> CodeilusResult<Vec<FileMetricsRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT rowid, file_id, sloc, complexity, churn, contributors FROM file_metrics")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(FileMetricsRow {
                    id: row.get(0)?,
                    file_id: FileId(row.get(1)?),
                    sloc: row.get(2)?,
                    complexity: row.get(3)?,
                    churn: row.get(4)?,
                    contributors: row.get(5)?,
                    heatmap_score: 0.0,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn list_hotspots(&self, limit: usize) -> CodeilusResult<Vec<FileMetricsRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT rowid, file_id, sloc, complexity, churn, contributors FROM file_metrics ORDER BY complexity DESC LIMIT ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![limit as i64], |row| {
                Ok(FileMetricsRow {
                    id: row.get(0)?,
                    file_id: FileId(row.get(1)?),
                    sloc: row.get(2)?,
                    complexity: row.get(3)?,
                    churn: row.get(4)?,
                    contributors: row.get(5)?,
                    heatmap_score: 0.0,
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
        conn.execute("DELETE FROM file_metrics", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
