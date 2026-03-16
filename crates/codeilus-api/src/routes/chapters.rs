//! Chapter API routes — learning path structure.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use std::sync::Arc;

use codeilus_core::ids::ChapterId;
use codeilus_db::{ChapterRepo, NarrativeRepo, QuizRepo};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct ChapterResponse {
    pub id: i64,
    pub order_index: i64,
    pub title: String,
    pub description: String,
    pub community_id: Option<i64>,
    pub difficulty: String,
    pub sections: Vec<SectionResponse>,
    /// Module summary narrative content (if available)
    pub narrative: Option<String>,
}

#[derive(Serialize)]
pub struct SectionResponse {
    pub id: i64,
    pub title: String,
    pub kind: String,
    pub content: String,
}

/// GET /api/v1/chapters — List all chapters with sections and narratives
async fn list_chapters(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cache_key = "chapters:all".to_string();
    if let Some(cached) = state.cache.json.get(&cache_key) {
        return Ok(Json(cached));
    }

    let chapter_repo = ChapterRepo::new(Arc::clone(&state.db));
    let narrative_repo = NarrativeRepo::new(Arc::clone(&state.db));

    let chapters = chapter_repo.list_ordered()?;
    let mut result = Vec::with_capacity(chapters.len());

    for ch in chapters {
        let sections = chapter_repo.list_sections(ch.id)?;
        let narrative = ch
            .community_id
            .and_then(|cid| {
                narrative_repo
                    .get_by_kind_and_target("module_summary", cid)
                    .ok()
                    .flatten()
            })
            .map(|n| n.content);

        result.push(ChapterResponse {
            id: ch.id.0,
            order_index: ch.order_index,
            title: ch.title,
            description: ch.description,
            community_id: ch.community_id,
            difficulty: ch.difficulty,
            sections: sections
                .into_iter()
                .map(|s| SectionResponse {
                    id: s.id,
                    title: s.title,
                    kind: s.kind,
                    content: s.content,
                })
                .collect(),
            narrative,
        });
    }

    let response = serde_json::to_value(&result)
        .map_err(|e| ApiError::from(codeilus_core::error::CodeilusError::Internal(e.to_string())))?;
    state.cache.json.insert(cache_key, response.clone());
    Ok(Json(response))
}

/// GET /api/v1/chapters/:id — Get single chapter with full detail
async fn get_chapter(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<ChapterResponse>, ApiError> {
    let chapter_repo = ChapterRepo::new(Arc::clone(&state.db));
    let narrative_repo = NarrativeRepo::new(Arc::clone(&state.db));

    let ch = chapter_repo.get(ChapterId(id))?;
    let sections = chapter_repo.list_sections(ch.id)?;
    let narrative = ch
        .community_id
        .and_then(|cid| {
            narrative_repo
                .get_by_kind_and_target("module_summary", cid)
                .ok()
                .flatten()
        })
        .map(|n| n.content);

    Ok(Json(ChapterResponse {
        id: ch.id.0,
        order_index: ch.order_index,
        title: ch.title,
        description: ch.description,
        community_id: ch.community_id,
        difficulty: ch.difficulty,
        sections: sections
            .into_iter()
            .map(|s| SectionResponse {
                id: s.id,
                title: s.title,
                kind: s.kind,
                content: s.content,
            })
            .collect(),
        narrative,
    }))
}

#[derive(Serialize)]
pub struct QuizQuestionResponse {
    pub id: i64,
    pub chapter_id: i64,
    pub question: String,
    pub options: Vec<String>,
    pub kind: String,
}

/// GET /api/v1/chapters/:id/quiz — Get quiz questions for a chapter
async fn get_chapter_quiz(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<QuizQuestionResponse>>, ApiError> {
    let quiz_repo = QuizRepo::new(Arc::clone(&state.db));
    let questions = quiz_repo.list_by_chapter(ChapterId(id))?;

    Ok(Json(
        questions
            .into_iter()
            .map(|q| QuizQuestionResponse {
                id: q.id,
                chapter_id: q.chapter_id,
                question: q.question,
                options: q.options,
                kind: q.kind,
            })
            .collect(),
    ))
}

#[derive(serde::Deserialize)]
pub struct QuizAnswerRequest {
    pub answer: String,
}

#[derive(Serialize)]
pub struct QuizAnswerResponse {
    pub correct: bool,
    pub xp_earned: i64,
    pub explanation: String,
}

/// POST /api/v1/quiz/:question_id/answer — Submit a quiz answer
async fn submit_quiz_answer(
    State(state): State<AppState>,
    Path(question_id): Path<i64>,
    Json(body): Json<QuizAnswerRequest>,
) -> Result<Json<QuizAnswerResponse>, ApiError> {
    let conn = state.db.connection();
    let result: Result<(String, i64, String, String), _> = conn.query_row(
        "SELECT COALESCE(options, '[]'), COALESCE(correct_index, 0), COALESCE(explanation, ''), kind FROM quiz_questions WHERE id = ?1",
        rusqlite::params![question_id],
        |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?, row.get(3)?)),
    );
    drop(conn);

    match result {
        Ok((options_json, correct_index, explanation, _kind)) => {
            let options: Vec<String> = serde_json::from_str(&options_json).unwrap_or_default();
            let correct_answer = options.get(correct_index as usize).cloned().unwrap_or_default();
            let is_correct = body.answer == correct_answer;
            let xp = if is_correct { 25 } else { 0 };

            Ok(Json(QuizAnswerResponse {
                correct: is_correct,
                xp_earned: xp,
                explanation,
            }))
        }
        Err(_) => Ok(Json(QuizAnswerResponse {
            correct: false,
            xp_earned: 0,
            explanation: "Question not found".to_string(),
        })),
    }
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/chapters", get(list_chapters))
        .route("/chapters/:id", get(get_chapter))
        .route("/chapters/:id/quiz", get(get_chapter_quiz))
        .route("/quiz/:question_id/answer", axum::routing::post(submit_quiz_answer))
}
