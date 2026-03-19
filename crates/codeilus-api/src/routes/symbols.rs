//! Symbol API routes.

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use std::sync::Arc;

use codeilus_core::ids::SymbolId;
use codeilus_db::{SymbolRepo, SymbolRow};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SymbolSearchQuery {
    pub q: Option<String>,
    pub kind: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// GET /api/v1/symbols — List symbols with pagination, optional ?kind= filter
async fn list_symbols(
    State(state): State<AppState>,
    Query(query): Query<SymbolSearchQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit = query.limit.unwrap_or(50).clamp(1, 200);
    let offset = query.offset.unwrap_or(0).max(0);
    let cache_key = format!("symbols:k={:?}:l={}:o={}", query.kind, limit, offset);

    if let Some(cached) = state.cache.json.get(&cache_key) {
        return Ok(Json(cached));
    }

    let repo = SymbolRepo::new(Arc::clone(&state.db));
    let results = repo.list_paginated(query.kind.as_deref(), limit, offset)?;
    let value = serde_json::to_value(&results)
        .map_err(|e| ApiError::from(codeilus_core::error::CodeilusError::Internal(e.to_string())))?;
    state.cache.json.insert(cache_key, value.clone());
    Ok(Json(value))
}

/// GET /api/v1/symbols/:id — Get a single symbol by ID
async fn get_symbol(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SymbolRow>, ApiError> {
    let repo = SymbolRepo::new(Arc::clone(&state.db));
    let symbol = repo.get(SymbolId(id))?;
    Ok(Json(symbol))
}

/// GET /api/v1/symbols/search?q=foo — Search symbols by name prefix
async fn search_symbols(
    State(state): State<AppState>,
    Query(query): Query<SymbolSearchQuery>,
) -> Result<Json<Vec<SymbolRow>>, ApiError> {
    let repo = SymbolRepo::new(Arc::clone(&state.db));
    let q = query.q.as_deref().unwrap_or("");
    let results = repo.search(q)?;
    Ok(Json(results))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/symbols", get(list_symbols))
        .route("/symbols/search", get(search_symbols))
        .route("/symbols/:id", get(get_symbol))
}
