//! Schematic API: unified tree + graph + learning endpoint.

use std::sync::Arc;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use codeilus_db::SchematicRepo;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SchematicQuery {
    pub depth: Option<u32>,
    pub community_id: Option<i64>,
    pub include_symbols: Option<bool>,
    pub include_edges: Option<bool>,
}

#[derive(Deserialize)]
pub struct ExpandQuery {
    pub node_id: String,
    pub include_symbols: Option<bool>,
    pub include_edges: Option<bool>,
}

#[derive(Deserialize)]
pub struct DetailQuery {
    pub node_id: String,
    pub include_source: Option<bool>,
}

async fn get_schematic(
    State(state): State<AppState>,
    Query(query): Query<SchematicQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let depth = query.depth.unwrap_or(2).min(10);
    let include_symbols = query.include_symbols.unwrap_or(false);
    let include_edges = query.include_edges.unwrap_or(false);

    let cache_key = format!(
        "schematic:d={}:c={:?}:s={}:e={}",
        depth, query.community_id, include_symbols, include_edges
    );

    if let Some(cached) = state.cache.json.get(&cache_key) {
        return Ok(Json(cached));
    }

    let repo = SchematicRepo::new(Arc::clone(&state.db));
    let result = repo.get_schematic(depth, query.community_id, include_symbols, include_edges)?;

    let value = serde_json::to_value(&result)
        .map_err(|e| ApiError::from(codeilus_core::CodeilusError::Internal(format!("serialize: {}", e))))?;
    state.cache.json.insert(cache_key, value.clone());

    Ok(Json(value))
}

async fn expand_node(
    State(state): State<AppState>,
    Query(query): Query<ExpandQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let include_symbols = query.include_symbols.unwrap_or(true);
    let include_edges = query.include_edges.unwrap_or(true);

    let repo = SchematicRepo::new(Arc::clone(&state.db));
    let result = repo.expand_node(&query.node_id, include_symbols, include_edges)?;

    let value = serde_json::to_value(&result)
        .map_err(|e| ApiError::from(codeilus_core::CodeilusError::Internal(format!("serialize: {}", e))))?;

    Ok(Json(value))
}

async fn get_detail(
    State(state): State<AppState>,
    Query(query): Query<DetailQuery>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let include_source = query.include_source.unwrap_or(true);

    let cache_key = format!("schematic_detail:{}:s={}", query.node_id, include_source);
    if let Some(cached) = state.cache.json.get(&cache_key) {
        return Ok(Json(cached));
    }

    let repo = SchematicRepo::new(Arc::clone(&state.db));
    let result = repo.get_detail(&query.node_id, include_source)?;

    let value = serde_json::to_value(&result)
        .map_err(|e| ApiError::from(codeilus_core::CodeilusError::Internal(format!("serialize: {}", e))))?;
    state.cache.json.insert(cache_key, value.clone());

    Ok(Json(value))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/schematic", get(get_schematic))
        .route("/schematic/expand", get(expand_node))
        .route("/schematic/detail", get(get_detail))
}
