//! File API routes.

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use std::sync::Arc;

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
    let repo = FileRepo::new(Arc::clone(&state.db));
    let files = repo.list(query.language.as_deref())?;
    Ok(Json(files))
}

/// GET /api/v1/files/:id — Get a single file by ID
async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<FileRow>, ApiError> {
    let repo = FileRepo::new(Arc::clone(&state.db));
    let file = repo.get(FileId(id))?;
    Ok(Json(file))
}

/// GET /api/v1/files/:id/symbols — List symbols for a file
async fn get_file_symbols(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<SymbolRow>>, ApiError> {
    let repo = SymbolRepo::new(Arc::clone(&state.db));
    let symbols = repo.list_by_file(FileId(id))?;
    Ok(Json(symbols))
}

#[derive(Deserialize)]
pub struct SourceQuery {
    pub start: Option<usize>,
    pub end: Option<usize>,
}

#[derive(Serialize)]
pub struct SourceResponse {
    pub path: String,
    pub language: Option<String>,
    pub lines: Vec<SourceLine>,
    pub total_lines: usize,
}

#[derive(Serialize)]
pub struct SourceLine {
    pub number: usize,
    pub content: String,
}

/// GET /api/v1/files/:id/source — Read file source code from disk
async fn get_file_source(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Query(query): Query<SourceQuery>,
) -> Result<Json<SourceResponse>, ApiError> {
    let repo = FileRepo::new(Arc::clone(&state.db));
    let file = repo.get(FileId(id))?;

    let repo_root = state.repo_root.as_ref().ok_or_else(|| ApiError {
        status: StatusCode::BAD_REQUEST,
        message: "No repository has been analyzed".to_string(),
    })?;

    // Resolve the file path relative to repo root
    let clean_path = file.path.strip_prefix("./").unwrap_or(&file.path);
    let full_path = repo_root.join(clean_path);

    // Canonicalize and verify the path stays within repo root (prevent path traversal)
    let canonical = full_path.canonicalize().map_err(|e| ApiError {
        status: StatusCode::NOT_FOUND,
        message: format!("Could not resolve file path: {}", e),
    })?;
    let canonical_root = repo_root.canonicalize().map_err(|e| ApiError {
        status: StatusCode::INTERNAL_SERVER_ERROR,
        message: format!("Could not resolve repo root: {}", e),
    })?;
    if !canonical.starts_with(&canonical_root) {
        return Err(ApiError {
            status: StatusCode::FORBIDDEN,
            message: "Path traversal detected".to_string(),
        });
    }

    let content = std::fs::read_to_string(&canonical).map_err(|e| {
        ApiError {
            status: StatusCode::NOT_FOUND,
            message: format!("Could not read file {}: {}", canonical.display(), e),
        }
    })?;

    let all_lines: Vec<&str> = content.lines().collect();
    let total_lines = all_lines.len();

    let start = query.start.unwrap_or(1).max(1);
    let end = query.end.unwrap_or(total_lines).min(total_lines);

    let lines: Vec<SourceLine> = all_lines
        .iter()
        .enumerate()
        .skip(start - 1)
        .take(end - start + 1)
        .map(|(i, line)| SourceLine {
            number: i + 1,
            content: line.to_string(),
        })
        .collect();

    Ok(Json(SourceResponse {
        path: file.path,
        language: file.language,
        lines,
        total_lines,
    }))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/files", get(list_files))
        .route("/files/:id", get(get_file))
        .route("/files/:id/symbols", get(get_file_symbols))
        .route("/files/:id/source", get(get_file_source))
}
