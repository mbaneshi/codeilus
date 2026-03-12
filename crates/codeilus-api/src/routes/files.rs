//! File API routes.

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use codeilus_core::ids::FileId;
use codeilus_db::{FileRepo, FileRow, SymbolRepo, SymbolRow};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct FileListQuery {
    pub language: Option<String>,
}

/// GET /api/v1/files — List all files, optional ?language= filter
async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<FileListQuery>,
) -> Result<Json<Vec<FileRow>>, ApiError> {
    let repo = FileRepo::new(state.db.conn_arc());
    let files = repo.list(query.language.as_deref())?;
    Ok(Json(files))
}

/// GET /api/v1/files/:id — Get a single file by ID
async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<FileRow>, ApiError> {
    let repo = FileRepo::new(state.db.conn_arc());
    let file = repo.get(FileId(id))?;
    Ok(Json(file))
}

/// GET /api/v1/files/:id/symbols — List symbols for a file
async fn get_file_symbols(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<SymbolRow>>, ApiError> {
    let repo = SymbolRepo::new(state.db.conn_arc());
    let symbols = repo.list_by_file(FileId(id))?;
    Ok(Json(symbols))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/files", get(list_files))
        .route("/files/:id", get(get_file))
        .route("/files/:id/symbols", get(get_file_symbols))
}
