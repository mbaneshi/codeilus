//! Narrative API routes — pre-generated LLM content.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

use std::sync::Arc;

use codeilus_db::NarrativeRepo;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct NarrativeResponse {
    pub id: i64,
    pub kind: String,
    pub target_id: Option<i64>,
    pub content: String,
    pub generated_at: String,
    pub is_placeholder: bool,
}

/// GET /api/v1/narratives — List all narratives
async fn list_narratives(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cache_key = "narratives:all".to_string();
    if let Some(cached) = state.cache.json.get(&cache_key) {
        return Ok(Json(cached));
    }

    let repo = NarrativeRepo::new(Arc::clone(&state.db));
    let rows = repo.list()?;

    let result: Vec<NarrativeResponse> = rows
        .into_iter()
        .map(|r| NarrativeResponse {
            id: r.id,
            kind: r.kind,
            target_id: r.target_id,
            content: r.content,
            generated_at: r.generated_at,
            is_placeholder: r.is_placeholder,
        })
        .collect();

    let response = serde_json::to_value(&result)
        .map_err(|e| ApiError::from(codeilus_core::error::CodeilusError::Internal(e.to_string())))?;
    state.cache.json.insert(cache_key, response.clone());
    Ok(Json(response))
}

/// GET /api/v1/narratives/:kind — Get narrative by kind (e.g., "overview", "architecture")
async fn get_by_kind(
    State(state): State<AppState>,
    Path(kind): Path<String>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cache_key = format!("narratives:kind={}", kind);
    if let Some(cached) = state.cache.json.get(&cache_key) {
        return Ok(Json(cached));
    }

    let repo = NarrativeRepo::new(Arc::clone(&state.db));
    let row = repo.get_by_kind(&kind)?;

    let result = row.map(|r| NarrativeResponse {
        id: r.id,
        kind: r.kind,
        target_id: r.target_id,
        content: r.content,
        generated_at: r.generated_at,
        is_placeholder: r.is_placeholder,
    });

    let response = serde_json::to_value(&result)
        .map_err(|e| ApiError::from(codeilus_core::error::CodeilusError::Internal(e.to_string())))?;
    state.cache.json.insert(cache_key, response.clone());
    Ok(Json(response))
}

/// GET /api/v1/narratives/:kind/:target_id — Get narrative by kind and target
async fn get_by_kind_and_target(
    State(state): State<AppState>,
    Path((kind, target_id)): Path<(String, i64)>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cache_key = format!("narratives:kind={}:target={}", kind, target_id);
    if let Some(cached) = state.cache.json.get(&cache_key) {
        return Ok(Json(cached));
    }

    let repo = NarrativeRepo::new(Arc::clone(&state.db));
    let row = repo.get_by_kind_and_target(&kind, target_id)?;

    let result = row.map(|r| NarrativeResponse {
        id: r.id,
        kind: r.kind,
        target_id: r.target_id,
        content: r.content,
        generated_at: r.generated_at,
        is_placeholder: r.is_placeholder,
    });

    let response = serde_json::to_value(&result)
        .map_err(|e| ApiError::from(codeilus_core::error::CodeilusError::Internal(e.to_string())))?;
    state.cache.json.insert(cache_key, response.clone());
    Ok(Json(response))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/narratives", get(list_narratives))
        .route("/narratives/:kind", get(get_by_kind))
        .route("/narratives/:kind/:target_id", get(get_by_kind_and_target))
}
