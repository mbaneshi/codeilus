//! Annotation API routes — user notes on graph nodes and edges.

use axum::extract::{Path, Query, State};
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use codeilus_db::AnnotationRepo;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Serialize)]
pub struct AnnotationResponse {
    pub id: i64,
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
    pub flagged: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Deserialize)]
pub struct ListQuery {
    pub flagged: Option<bool>,
}

#[derive(Deserialize)]
pub struct CreateRequest {
    pub target_type: String,
    pub target_id: i64,
    pub content: String,
}

#[derive(Deserialize)]
pub struct UpdateRequest {
    pub content: String,
}

fn to_response(r: codeilus_db::AnnotationRow) -> AnnotationResponse {
    AnnotationResponse {
        id: r.id,
        target_type: r.target_type,
        target_id: r.target_id,
        content: r.content,
        flagged: r.flagged,
        created_at: r.created_at,
        updated_at: r.updated_at,
    }
}

/// GET /api/v1/annotations
async fn list_annotations(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<AnnotationResponse>>, ApiError> {
    let repo = AnnotationRepo::new(state.db.conn_arc());
    let rows = if query.flagged.unwrap_or(false) {
        repo.list_flagged()?
    } else {
        repo.list_all()?
    };
    Ok(Json(rows.into_iter().map(to_response).collect()))
}

/// GET /api/v1/annotations/:target_type/:target_id
async fn list_by_target(
    State(state): State<AppState>,
    Path((target_type, target_id)): Path<(String, i64)>,
) -> Result<Json<Vec<AnnotationResponse>>, ApiError> {
    let repo = AnnotationRepo::new(state.db.conn_arc());
    let rows = repo.list_by_target(&target_type, target_id)?;
    Ok(Json(rows.into_iter().map(to_response).collect()))
}

/// POST /api/v1/annotations
async fn create_annotation(
    State(state): State<AppState>,
    Json(body): Json<CreateRequest>,
) -> Result<Json<AnnotationResponse>, ApiError> {
    let repo = AnnotationRepo::new(state.db.conn_arc());
    let id = repo.insert(&body.target_type, body.target_id, &body.content)?;
    let rows = repo.list_by_target(&body.target_type, body.target_id)?;
    let row = rows
        .into_iter()
        .find(|r| r.id == id)
        .ok_or_else(|| codeilus_core::CodeilusError::NotFound("annotation".to_string()))?;
    Ok(Json(to_response(row)))
}

/// PUT /api/v1/annotations/:id
async fn update_annotation(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateRequest>,
) -> Result<Json<()>, ApiError> {
    let repo = AnnotationRepo::new(state.db.conn_arc());
    repo.update(id, &body.content)?;
    Ok(Json(()))
}

/// POST /api/v1/annotations/:id/flag
async fn toggle_flag(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let repo = AnnotationRepo::new(state.db.conn_arc());
    let flagged = repo.toggle_flag(id)?;
    Ok(Json(serde_json::json!({ "flagged": flagged })))
}

/// DELETE /api/v1/annotations/:id
async fn delete_annotation(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<()>, ApiError> {
    let repo = AnnotationRepo::new(state.db.conn_arc());
    repo.delete(id)?;
    Ok(Json(()))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/annotations", get(list_annotations).post(create_annotation))
        .route(
            "/annotations/{id}",
            put(update_annotation).delete(axum::routing::delete(delete_annotation)),
        )
        .route("/annotations/{id}/flag", post(toggle_flag))
        .route(
            "/annotations/{target_type}/{target_id}",
            get(list_by_target),
        )
}
