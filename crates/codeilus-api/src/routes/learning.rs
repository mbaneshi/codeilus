//! Learning API routes — progress tracking, section completion, learner stats.

use axum::extract::{Path, State};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Serialize;

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
    let conn = state.db.conn_arc();
    let conn_guard = conn.lock().expect("db mutex poisoned");

    let mut stmt = conn_guard
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
    let conn = state.db.conn_arc();
    let progress_repo = ProgressRepo::new(conn.clone());

    // Look up the content_type for this section_id so we can use record_section
    let content_type: String = {
        let conn_guard = conn.lock().expect("db mutex poisoned");
        conn_guard
            .query_row(
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
    let conn = state.db.conn_arc();
    let progress_repo = ProgressRepo::new(conn.clone());

    let stats = progress_repo.get_or_create_stats()?;
    let chapters_completed = progress_repo.count_completed_chapters()?;

    // Fetch full badge records from DB
    let badges = {
        let conn_guard = conn.lock().expect("db mutex poisoned");
        let mut stmt = conn_guard
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

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/progress", get(list_progress))
        .route(
            "/chapters/:chapter_id/sections/:section_id/complete",
            post(mark_section_complete),
        )
        .route("/learner/stats", get(get_learner_stats))
}
