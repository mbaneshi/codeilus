//! Narrative API routes — pre-generated LLM content.

use axum::extract::{Path, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Serialize;

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
) -> Result<Json<Vec<NarrativeResponse>>, ApiError> {
    let repo = NarrativeRepo::new(state.db.conn_arc());
    let rows = repo.list()?;

    Ok(Json(
        rows.into_iter()
            .map(|r| NarrativeResponse {
                id: r.id,
                kind: r.kind,
                target_id: r.target_id,
                content: r.content,
                generated_at: r.generated_at,
                is_placeholder: r.is_placeholder,
            })
            .collect(),
    ))
}

/// GET /api/v1/narratives/:kind — Get narrative by kind (e.g., "overview", "architecture")
async fn get_by_kind(
    State(state): State<AppState>,
    Path(kind): Path<String>,
) -> Result<Json<Option<NarrativeResponse>>, ApiError> {
    let repo = NarrativeRepo::new(state.db.conn_arc());
    let row = repo.get_by_kind(&kind)?;

    Ok(Json(row.map(|r| NarrativeResponse {
        id: r.id,
        kind: r.kind,
        target_id: r.target_id,
        content: r.content,
        generated_at: r.generated_at,
        is_placeholder: r.is_placeholder,
    })))
}

/// GET /api/v1/narratives/:kind/:target_id — Get narrative by kind and target
async fn get_by_kind_and_target(
    State(state): State<AppState>,
    Path((kind, target_id)): Path<(String, i64)>,
) -> Result<Json<Option<NarrativeResponse>>, ApiError> {
    let repo = NarrativeRepo::new(state.db.conn_arc());
    let row = repo.get_by_kind_and_target(&kind, target_id)?;

    Ok(Json(row.map(|r| NarrativeResponse {
        id: r.id,
        kind: r.kind,
        target_id: r.target_id,
        content: r.content,
        generated_at: r.generated_at,
        is_placeholder: r.is_placeholder,
    })))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/narratives", get(list_narratives))
        .route("/narratives/:kind", get(get_by_kind))
        .route("/narratives/:kind/:target_id", get(get_by_kind_and_target))
}
