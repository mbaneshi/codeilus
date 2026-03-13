//! Repository for progress tracking, learner stats, and badge storage.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressRow {
    pub id: i64,
    pub chapter_id: i64,
    pub section_id: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerStatsRow {
    pub total_xp: i64,
    pub streak_days: i64,
    pub last_active: String,
}

pub struct ProgressRepo {
    conn: Arc<Mutex<Connection>>,
}

impl ProgressRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Record a section completion. Returns the progress row id.
    pub fn record_section(&self, chapter_id: i64, section_id: &str) -> CodeilusResult<i64> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        // Look up the chapter_sections.id by chapter_id and content_type
        let cs_id: i64 = conn
            .query_row(
                "SELECT id FROM chapter_sections WHERE chapter_id = ?1 AND content_type = ?2",
                params![chapter_id, section_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => CodeilusError::NotFound(format!(
                    "section '{}' in chapter {}",
                    section_id, chapter_id
                )),
                other => CodeilusError::Database(Box::new(other)),
            })?;

        conn.execute(
            "INSERT OR REPLACE INTO progress (chapter_id, section_id, completed, completed_at) VALUES (?1, ?2, 1, datetime('now'))",
            params![chapter_id, cs_id],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(conn.last_insert_rowid())
    }

    /// Record a quiz attempt. If passed, marks the quiz section as completed.
    pub fn record_quiz_attempt(
        &self,
        chapter_id: i64,
        _score: f64,
        passed: bool,
    ) -> CodeilusResult<i64> {
        if passed {
            self.record_section(chapter_id, "quiz")
        } else {
            Ok(0)
        }
    }

    /// Check if a section is completed.
    pub fn is_section_complete(&self, chapter_id: i64, section_id: &str) -> CodeilusResult<bool> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        // Look up chapter_sections.id
        let cs_id: Result<i64, _> = conn.query_row(
            "SELECT id FROM chapter_sections WHERE chapter_id = ?1 AND content_type = ?2",
            params![chapter_id, section_id],
            |row| row.get(0),
        );
        let cs_id = match cs_id {
            Ok(id) => id,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(false),
            Err(e) => return Err(CodeilusError::Database(Box::new(e))),
        };

        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM progress WHERE chapter_id = ?1 AND section_id = ?2 AND completed = 1",
                params![chapter_id, cs_id],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(count > 0)
    }

    /// Get chapter progress as fraction (0.0 to 1.0).
    pub fn get_chapter_progress(&self, chapter_id: i64) -> CodeilusResult<f64> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let total: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM chapter_sections WHERE chapter_id = ?1",
                params![chapter_id],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        if total == 0 {
            return Ok(0.0);
        }
        let completed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM progress WHERE chapter_id = ?1 AND completed = 1",
                params![chapter_id],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(completed as f64 / total as f64)
    }

    /// Get overall progress across all chapters.
    pub fn get_overall_progress(&self) -> CodeilusResult<f64> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let total: i64 = conn
            .query_row("SELECT COUNT(*) FROM chapter_sections", [], |row| {
                row.get(0)
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        if total == 0 {
            return Ok(0.0);
        }
        let completed: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM progress WHERE completed = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(completed as f64 / total as f64)
    }

    /// Get or create the learner stats row.
    pub fn get_or_create_stats(&self) -> CodeilusResult<LearnerStatsRow> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let result = conn.query_row(
            "SELECT total_xp, streak_days, last_active FROM learner_stats WHERE id = 1",
            [],
            |row| {
                Ok(LearnerStatsRow {
                    total_xp: row.get(0)?,
                    streak_days: row.get(1)?,
                    last_active: row.get::<_, Option<String>>(2)?.unwrap_or_default(),
                })
            },
        );
        match result {
            Ok(stats) => Ok(stats),
            Err(rusqlite::Error::QueryReturnedNoRows) => {
                conn.execute(
                    "INSERT INTO learner_stats (id, total_xp, streak_days, last_active) VALUES (1, 0, 0, datetime('now'))",
                    [],
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
                Ok(LearnerStatsRow {
                    total_xp: 0,
                    streak_days: 0,
                    last_active: String::new(),
                })
            }
            Err(e) => Err(CodeilusError::Database(Box::new(e))),
        }
    }

    /// Update learner stats.
    pub fn update_stats(&self, stats: &LearnerStatsRow) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute(
            "UPDATE learner_stats SET total_xp = ?1, streak_days = ?2, last_active = ?3 WHERE id = 1",
            params![stats.total_xp, stats.streak_days, stats.last_active],
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    /// List completed section IDs (content_types) for a chapter.
    pub fn list_completed_sections(&self, chapter_id: i64) -> CodeilusResult<Vec<String>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare(
                "SELECT cs.content_type FROM progress p \
                 JOIN chapter_sections cs ON cs.id = p.section_id \
                 WHERE p.chapter_id = ?1 AND p.completed = 1",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![chapter_id], |row| row.get(0))
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Count how many quizzes have been passed (chapters with "quiz" section completed).
    pub fn count_quizzes_passed(&self) -> CodeilusResult<usize> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(DISTINCT p.chapter_id) FROM progress p \
                 JOIN chapter_sections cs ON cs.id = p.section_id \
                 WHERE cs.content_type = 'quiz' AND p.completed = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(count as usize)
    }

    /// Insert a badge if it doesn't already exist.
    pub fn insert_badge(&self, name: &str, description: &str) -> CodeilusResult<bool> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let result = conn.execute(
            "INSERT OR IGNORE INTO badges (name, description, earned_at) VALUES (?1, ?2, datetime('now'))",
            params![name, description],
        );
        match result {
            Ok(changes) => Ok(changes > 0),
            Err(e) => Err(CodeilusError::Database(Box::new(e))),
        }
    }

    /// List all earned badge names.
    pub fn list_badges(&self) -> CodeilusResult<Vec<String>> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let mut stmt = conn
            .prepare("SELECT name FROM badges WHERE earned_at IS NOT NULL")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| row.get(0))
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
        }
        Ok(result)
    }

    /// Count completed chapters (all sections done).
    pub fn count_completed_chapters(&self) -> CodeilusResult<usize> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        // A chapter is complete if all its sections have corresponding progress entries
        let mut stmt = conn
            .prepare(
                "SELECT c.id, \
                    (SELECT COUNT(*) FROM chapter_sections cs WHERE cs.chapter_id = c.id) AS total, \
                    (SELECT COUNT(*) FROM progress p WHERE p.chapter_id = c.id AND p.completed = 1) AS done \
                 FROM chapters c",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                let total: i64 = row.get(1)?;
                let done: i64 = row.get(2)?;
                Ok((total, done))
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut count = 0;
        for row in rows {
            let (total, done) = row.map_err(|e| CodeilusError::Database(Box::new(e)))?;
            if total > 0 && done >= total {
                count += 1;
            }
        }
        Ok(count)
    }

    /// Count total completed sections across all chapters.
    pub fn count_completed_sections(&self) -> CodeilusResult<usize> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        let count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM progress WHERE completed = 1",
                [],
                |row| row.get(0),
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(count as usize)
    }

    /// Check if a specific chapter (by order_index) is fully complete.
    pub fn is_chapter_complete(&self, chapter_id: i64) -> CodeilusResult<bool> {
        let progress = self.get_chapter_progress(chapter_id)?;
        Ok((progress - 1.0).abs() < f64::EPSILON)
    }

    pub fn delete_all(&self) -> CodeilusResult<()> {
        let conn = self.conn.lock().expect("db mutex poisoned");
        conn.execute("DELETE FROM progress", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        conn.execute("DELETE FROM badges", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        conn.execute("DELETE FROM learner_stats", [])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}
