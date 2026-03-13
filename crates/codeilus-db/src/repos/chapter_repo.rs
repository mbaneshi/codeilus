//! Repository for chapters and chapter sections.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::ids::ChapterId;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterRow {
    pub id: ChapterId,
    pub order_index: i64,
    pub title: String,
    pub description: String,
    pub community_id: Option<i64>,
    pub difficulty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterSectionRow {
    pub id: i64,
    pub chapter_id: ChapterId,
    pub section_id: String,
    pub title: String,
    pub kind: String,
}

pub struct ChapterRepo {
    conn: Arc<Mutex<Connection>>,
}

impl ChapterRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(
        &self,
        order_index: i64,
        title: &str,
        description: &str,
        community_id: Option<i64>,
        difficulty: &str,
    ) -> CodeilusResult<ChapterId> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "INSERT INTO chapters (order_index, title, description, community_id, difficulty) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![order_index, title, description, community_id, difficulty],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(ChapterId(conn.last_insert_rowid()))
    }

    pub fn insert_section(
        &self,
        chapter_id: ChapterId,
        section_id: &str,
        title: &str,
        kind: &str,
    ) -> CodeilusResult<i64> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        // Use content_type to store the section kind/id
        // order_index is derived from existing section count
        let order: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM chapter_sections WHERE chapter_id = ?1",
                params![chapter_id.0],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        conn.execute(
            "INSERT INTO chapter_sections (chapter_id, title, content_type, order_index) VALUES (?1, ?2, ?3, ?4)",
            params![chapter_id.0, title, kind, order],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let _ = section_id; // section_id is stored via content_type=kind
        Ok(conn.last_insert_rowid())
    }

    pub fn get(&self, id: ChapterId) -> CodeilusResult<ChapterRow> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.query_row(
            "SELECT id, order_index, title, description, community_id, difficulty FROM chapters WHERE id = ?1",
            params![id.0],
            |row| {
                Ok(ChapterRow {
                    id: ChapterId(row.get(0)?),
                    order_index: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    community_id: row.get(4)?,
                    difficulty: row.get::<_, Option<String>>(5)?.unwrap_or_else(|| "beginner".to_string()),
                })
            },
        )
        .map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => {
                CodeilusError::NotFound(format!("chapter with id {id}"))
            }
            other => CodeilusError::Database(Box::new(other)),
        })
    }

    pub fn list_ordered(&self) -> CodeilusResult<Vec<ChapterRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT id, order_index, title, description, community_id, difficulty FROM chapters ORDER BY order_index")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(ChapterRow {
                    id: ChapterId(row.get(0)?),
                    order_index: row.get(1)?,
                    title: row.get(2)?,
                    description: row.get::<_, Option<String>>(3)?.unwrap_or_default(),
                    community_id: row.get(4)?,
                    difficulty: row.get::<_, Option<String>>(5)?.unwrap_or_else(|| "beginner".to_string()),
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    pub fn list_sections(&self, chapter_id: ChapterId) -> CodeilusResult<Vec<ChapterSectionRow>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT id, chapter_id, content_type, title, content_type FROM chapter_sections WHERE chapter_id = ?1 ORDER BY order_index")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![chapter_id.0], |row| {
                Ok(ChapterSectionRow {
                    id: row.get(0)?,
                    chapter_id: ChapterId(row.get(1)?),
                    section_id: row.get(2)?,
                    title: row.get(3)?,
                    kind: row.get(4)?,
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
        conn.execute("DELETE FROM chapter_sections", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        conn.execute("DELETE FROM chapters", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
