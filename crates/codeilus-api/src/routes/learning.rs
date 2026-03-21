//! Learning API routes — progress tracking, section completion, learner stats.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;

use std::sync::Arc;

use codeilus_db::ProgressRepo;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct ProgressResponse {
    pub chapter_id: i64,
    pub section_id: i64,
    pub completed: bool,
    pub completed_at: Option<String>,
}

/// GET /api/v1/progress — List all progress records
async fn list_progress(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProgressResponse>>, ApiError> {
    let conn = state.db.connection();

    let mut stmt = conn
        .prepare(
            "SELECT chapter_id, section_id, completed, completed_at FROM progress WHERE completed = 1",
        )
        .map_err(|e| codeilus_core::CodeilusError::Database(Box::new(e)))?;

    let rows = stmt
        .query_map([], |row| {
            Ok(ProgressResponse {
                chapter_id: row.get(0)?,
                section_id: row.get(1)?,
                completed: row.get::<_, i64>(2)? != 0,
                completed_at: row.get(3)?,
            })
        })
        .map_err(|e| codeilus_core::CodeilusError::Database(Box::new(e)))?;

    let mut result = Vec::new();
    for row in rows {
        result.push(row.map_err(|e| codeilus_core::CodeilusError::Database(Box::new(e)))?);
    }

    Ok(Json(result))
}

#[derive(Serialize)]
pub struct SectionCompleteResponse {
    pub xp_earned: i64,
}

/// POST /api/v1/chapters/:chapter_id/sections/:section_id/complete
///
/// Marks a section as complete, awards 10 XP, and checks for chapter completion bonus (50 XP).
async fn mark_section_complete(
    State(state): State<AppState>,
    Path((chapter_id, section_id)): Path<(i64, i64)>,
) -> Result<Json<SectionCompleteResponse>, ApiError> {
    let progress_repo = ProgressRepo::new(Arc::clone(&state.db));

    // Look up the content_type for this section_id so we can use record_section
    let content_type: String = {
        let conn = state.db.connection();
        conn.query_row(
                "SELECT content_type FROM chapter_sections WHERE id = ?1 AND chapter_id = ?2",
                rusqlite::params![section_id, chapter_id],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => {
                    codeilus_core::CodeilusError::NotFound(format!(
                        "section {} in chapter {}",
                        section_id, chapter_id
                    ))
                }
                other => codeilus_core::CodeilusError::Database(Box::new(other)),
            })?
    };

    // Record section completion
    progress_repo.record_section(chapter_id, &content_type)?;

    // Award base XP
    let mut xp_earned: i64 = 10;

    // Check if the entire chapter is now complete for bonus XP
    if progress_repo.is_chapter_complete(chapter_id)? {
        xp_earned += 50;
    }

    // Update learner stats with earned XP
    let mut stats = progress_repo.get_or_create_stats()?;
    stats.total_xp += xp_earned;
    progress_repo.update_stats(&stats)?;

    Ok(Json(SectionCompleteResponse { xp_earned }))
}

#[derive(Serialize)]
pub struct BadgeResponse {
    pub id: i64,
    pub name: String,
    pub description: String,
    pub icon: String,
    pub earned_at: String,
}

#[derive(Serialize)]
pub struct LearnerStatsResponse {
    pub total_xp: i64,
    pub streak_days: i64,
    pub last_active: String,
    pub chapters_completed: usize,
    pub badges: Vec<BadgeResponse>,
}

/// GET /api/v1/learner/stats — Returns learner stats (XP, streak, badges)
async fn get_learner_stats(
    State(state): State<AppState>,
) -> Result<Json<LearnerStatsResponse>, ApiError> {
    let progress_repo = ProgressRepo::new(Arc::clone(&state.db));

    let stats = progress_repo.get_or_create_stats()?;
    let chapters_completed = progress_repo.count_completed_chapters()?;

    // Fetch full badge records from DB
    let badges = {
        let conn = state.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT id, name, COALESCE(description, ''), COALESCE(icon, ''), COALESCE(earned_at, '') FROM badges WHERE earned_at IS NOT NULL",
            )
            .map_err(|e| codeilus_core::CodeilusError::Database(Box::new(e)))?;

        let rows = stmt
            .query_map([], |row| {
                Ok(BadgeResponse {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    description: row.get(2)?,
                    icon: row.get(3)?,
                    earned_at: row.get(4)?,
                })
            })
            .map_err(|e| codeilus_core::CodeilusError::Database(Box::new(e)))?;

        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| codeilus_core::CodeilusError::Database(Box::new(e)))?);
        }
        result
    };

    Ok(Json(LearnerStatsResponse {
        total_xp: stats.total_xp,
        streak_days: stats.streak_days,
        last_active: stats.last_active,
        chapters_completed,
        badges,
    }))
}

#[derive(Serialize)]
pub struct SkipChapterResponse {
    pub sections_skipped: usize,
}

/// POST /api/v1/chapters/:id/skip
///
/// Marks all sections of a chapter as complete with xp_earned = 0 (skip/test-out).
async fn skip_chapter(
    State(state): State<AppState>,
    Path(chapter_id): Path<i64>,
) -> Result<Json<SkipChapterResponse>, ApiError> {
    let progress_repo = ProgressRepo::new(Arc::clone(&state.db));

    // Get all sections for this chapter
    let sections: Vec<(i64, String)> = {
        let conn = state.db.connection();
        let mut stmt = conn
            .prepare("SELECT id, content_type FROM chapter_sections WHERE chapter_id = ?1")
            .map_err(|e| codeilus_core::CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(rusqlite::params![chapter_id], |row| {
                Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|e| codeilus_core::CodeilusError::Database(Box::new(e)))?;
        let mut result = Vec::new();
        for row in rows {
            result.push(row.map_err(|e| codeilus_core::CodeilusError::Database(Box::new(e)))?);
        }
        result
    };

    if sections.is_empty() {
        return Err(codeilus_core::CodeilusError::NotFound(format!(
            "chapter {}",
            chapter_id
        )).into());
    }

    // Batch: get already-completed sections in one query, then only insert missing
    let completed = progress_repo.list_completed_sections(chapter_id)?;
    let completed_set: std::collections::HashSet<&str> = completed.iter().map(|s| s.as_str()).collect();

    let mut skipped = 0;
    for (_section_id, content_type) in &sections {
        if !completed_set.contains(content_type.as_str()) {
            progress_repo.record_section(chapter_id, content_type)?;
            skipped += 1;
        }
    }

    Ok(Json(SkipChapterResponse {
        sections_skipped: skipped,
    }))
}

/// DELETE /api/v1/progress
///
/// Deletes all progress records, badges, and learner stats.
async fn reset_progress(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let progress_repo = ProgressRepo::new(Arc::clone(&state.db));
    progress_repo.delete_all()?;
    Ok(Json(serde_json::json!({ "status": "ok" })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/progress", get(list_progress).delete(reset_progress))
        .route(
            "/chapters/:chapter_id/sections/:section_id/complete",
            post(mark_section_complete),
        )
        .route("/chapters/:chapter_id/skip", post(skip_chapter))
        .route("/learner/stats", get(get_learner_stats))
}
