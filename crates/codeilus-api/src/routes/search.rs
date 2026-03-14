//! Search API route.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use codeilus_search::{SearchEngine, SearchResult, SearchResultKind};

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SearchParams {
    q: String,
    #[serde(rename = "type")]
    kind: Option<String>,
    limit: Option<usize>,
}

/// GET /search?q=...&type=file|symbol|narrative&limit=20
async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<Vec<SearchResult>>, ApiError> {
    let engine = SearchEngine::new(Arc::clone(&state.db));
    let kind = params.kind.as_deref().map(|k| match k {
        "file" => SearchResultKind::File,
        "symbol" => SearchResultKind::Symbol,
        "narrative" => SearchResultKind::Narrative,
        _ => SearchResultKind::Symbol,
    });
    let limit = params.limit.unwrap_or(20).min(100);
    let results = engine.search(&params.q, kind, limit)?;
    Ok(Json(results))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/search", get(search))
}
