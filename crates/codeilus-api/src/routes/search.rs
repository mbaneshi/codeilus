//! Search API route.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

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

#[derive(Serialize)]
pub struct SearchResponse {
    results: Vec<SearchResultItem>,
}

#[derive(Serialize)]
pub struct SearchResultItem {
    kind: String,
    id: i64,
    name: String,
    path: Option<String>,
    score: f64,
    snippet: String,
}

impl From<SearchResult> for SearchResultItem {
    fn from(r: SearchResult) -> Self {
        let kind = match r.kind {
            SearchResultKind::File => "file",
            SearchResultKind::Symbol => "symbol",
            SearchResultKind::Narrative => "narrative",
        };
        Self {
            kind: kind.to_string(),
            id: r.id,
            name: r.name,
            path: r.metadata.file_path,
            score: r.score,
            snippet: r.snippet,
        }
    }
}

/// GET /api/v1/search?q=...&type=file|symbol|narrative&limit=20
async fn search(
    State(state): State<AppState>,
    Query(params): Query<SearchParams>,
) -> Result<Json<SearchResponse>, ApiError> {
    let engine = SearchEngine::new(Arc::clone(&state.db));
    let kind = params.kind.as_deref().map(|k| match k {
        "file" => SearchResultKind::File,
        "symbol" => SearchResultKind::Symbol,
        "narrative" => SearchResultKind::Narrative,
        _ => SearchResultKind::Symbol,
    });
    let limit = params.limit.unwrap_or(20).min(100);
    let results = engine.search(&params.q, kind, limit)?;
    let items = results.into_iter().map(SearchResultItem::from).collect();
    Ok(Json(SearchResponse { results: items }))
}

pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/search", get(search))
}
