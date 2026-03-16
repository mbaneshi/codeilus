//! Repository for quiz questions and attempts.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::ids::ChapterId;
use rusqlite::params;
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::pool::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestionRow {
    pub id: i64,
    pub chapter_id: i64,
    pub question: String,
    pub kind: String,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub explanation: String,
}

pub struct QuizRepo {
    db: Arc<DbPool>,
}

impl QuizRepo {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    pub fn insert(
        &self,
        chapter_id: ChapterId,
        question: &str,
        kind: &str,
        options: &[String],
        correct_index: usize,
        explanation: &str,
    ) -> CodeilusResult<i64> {
        let conn = self.db.connection();
        let options_json =
            serde_json::to_string(options).unwrap_or_else(|_| "[]".to_string());
        let answer = options
            .get(correct_index)
            .cloned()
            .unwrap_or_default();

        conn.execute(
            "INSERT INTO quiz_questions (chapter_id, question, answer, kind, options, correct_index, explanation) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![chapter_id.0, question, answer, kind, options_json, correct_index as i64, explanation],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(conn.last_insert_rowid())
    }

    pub fn list_by_chapter(&self, chapter_id: ChapterId) -> CodeilusResult<Vec<QuizQuestionRow>> {
        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, chapter_id, question, kind, COALESCE(options, '[]'), COALESCE(correct_index, 0), COALESCE(explanation, '') FROM quiz_questions WHERE chapter_id = ?1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        let rows = stmt
            .query_map(params![chapter_id.0], |row| {
                let options_json: String = row.get(4)?;
                let options: Vec<String> =
                    serde_json::from_str(&options_json).unwrap_or_default();
                let correct_index: i64 = row.get(5)?;
                Ok(QuizQuestionRow {
                    id: row.get(0)?,
                    chapter_id: row.get(1)?,
                    question: row.get(2)?,
                    kind: row.get(3)?,
                    options,
                    correct_index: correct_index as usize,
                    explanation: row.get(6)?,
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
        conn.execute("DELETE FROM quiz_attempts", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        conn.execute("DELETE FROM quiz_questions", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
