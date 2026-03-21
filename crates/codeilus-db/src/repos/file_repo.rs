use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::ids::FileId;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::pool::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRow {
    pub id: FileId,
    pub path: String,
    pub language: Option<String>,
    pub sloc: i64,
    pub last_modified: Option<String>,
}

pub struct FileRepo {
    db: Arc<DbPool>,
}

impl FileRepo {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// Insert a single file. Returns the new FileId.
    pub fn insert(
        &self,
        path: &str,
        language: Option<&str>,
        sloc: i64,
    ) -> CodeilusResult<FileId> {
        let conn = self.db.connection();
        conn.execute(
            "INSERT INTO files (path, language, sloc) VALUES (?1, ?2, ?3)",
            params![path, language, sloc],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(FileId(conn.last_insert_rowid()))
    }

    /// Batch insert files in a transaction. Returns all FileIds.
    pub fn insert_batch(
        &self,
        files: &[(String, Option<String>, i64)],
    ) -> CodeilusResult<Vec<FileId>> {
        let conn = self.db.connection();
        let tx = conn
            .unchecked_transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(files.len());
        {
            let mut stmt = tx
                .prepare("INSERT INTO files (path, language, sloc) VALUES (?1, ?2, ?3)")
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for (path, language, sloc) in files {
                stmt.execute(params![path, language, sloc])
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                ids.push(FileId(tx.last_insert_rowid()));
            }
        }
        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(ids)
    }

    /// Get a file by ID.
    pub fn get(&self, id: FileId) -> CodeilusResult<FileRow> {
        let conn = self.db.connection();
        conn.query_row(
            "SELECT id, path, language, COALESCE(sloc, 0), last_modified FROM files WHERE id = ?1",
            params![id.0],
            |row| {
                Ok(FileRow {
                    id: FileId(row.get(0)?),
                    path: row.get(1)?,
                    language: row.get(2)?,
                    sloc: row.get(3)?,
                    last_modified: row.get(4)?,
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CodeilusError::NotFound(format!("file with id {id}"))
            }
            other => CodeilusError::Database(Box::new(other)),
        })
    }

    /// Get a file by path.
    pub fn get_by_path(&self, path: &str) -> CodeilusResult<Option<FileRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("SELECT id, path, language, COALESCE(sloc, 0), last_modified FROM files WHERE path = ?1")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut rows = stmt
            .query_map(params![path], |row| {
                Ok(FileRow {
                    id: FileId(row.get(0)?),
                    path: row.get(1)?,
                    language: row.get(2)?,
                    sloc: row.get(3)?,
                    last_modified: row.get(4)?,
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

    /// List all files. Optional language filter.
    pub fn list(&self, language: Option<&str>) -> CodeilusResult<Vec<FileRow>> {
        let conn = self.db.connection();
        let mut result = Vec::new();
        match language {
            Some(lang) => {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, path, language, COALESCE(sloc, 0), last_modified FROM files WHERE language = ?1",
                    )
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                let rows = stmt
                    .query_map(params![lang], |row| {
                        Ok(FileRow {
                            id: FileId(row.get(0)?),
                            path: row.get(1)?,
                            language: row.get(2)?,
                            sloc: row.get(3)?,
                            last_modified: row.get(4)?,
                        })
                    })
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                for row in rows {
                    result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
                }
            }
            None => {
                let mut stmt = conn
                    .prepare("SELECT id, path, language, COALESCE(sloc, 0), last_modified FROM files")
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                let rows = stmt
                    .query_map([], |row| {
                        Ok(FileRow {
                            id: FileId(row.get(0)?),
                            path: row.get(1)?,
                            language: row.get(2)?,
                            sloc: row.get(3)?,
                            last_modified: row.get(4)?,
                        })
                    })
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                for row in rows {
                    result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
                }
            }
        }
        Ok(result)
    }

    /// List files with pagination. Optional language filter.
    pub fn list_paginated(
        &self,
        language: Option<&str>,
        limit: i64,
        offset: i64,
    ) -> CodeilusResult<Vec<FileRow>> {
        let conn = self.db.connection();
        let mut result = Vec::new();
        match language {
            Some(lang) => {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, path, language, COALESCE(sloc, 0), last_modified FROM files WHERE language = ?1 LIMIT ?2 OFFSET ?3",
                    )
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                let rows = stmt
                    .query_map(params![lang, limit, offset], |row| {
                        Ok(FileRow {
                            id: FileId(row.get(0)?),
                            path: row.get(1)?,
                            language: row.get(2)?,
                            sloc: row.get(3)?,
                            last_modified: row.get(4)?,
                        })
                    })
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                for row in rows {
                    result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
                }
            }
            None => {
                let mut stmt = conn
                    .prepare(
                        "SELECT id, path, language, COALESCE(sloc, 0), last_modified FROM files LIMIT ?1 OFFSET ?2",
                    )
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                let rows = stmt
                    .query_map(params![limit, offset], |row| {
                        Ok(FileRow {
                            id: FileId(row.get(0)?),
                            path: row.get(1)?,
                            language: row.get(2)?,
                            sloc: row.get(3)?,
                            last_modified: row.get(4)?,
                        })
                    })
                    .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                for row in rows {
                    result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
                }
            }
        }
        Ok(result)
    }

    /// Count total files.
    pub fn count(&self) -> CodeilusResult<usize> {
        let conn = self.db.connection();
        let count: i64 = conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(count as usize)
    }

    /// List all existing files as (path, last_modified) pairs for incremental parsing.
    pub fn list_existing(&self) -> CodeilusResult<Vec<(String, Option<String>)>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare("SELECT path, last_modified FROM files")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| Ok((row.get(0)?, row.get(1)?)))
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Delete all files (for re-analysis).
    pub fn delete_all(&self) -> CodeilusResult<()> {
        let conn = self.db.connection();
        conn.execute("DELETE FROM files", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
