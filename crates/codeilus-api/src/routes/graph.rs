//! Graph, community, and process API routes.

use std::collections::HashSet;

use axum::extract::{Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use codeilus_core::error::CodeilusError;
use rusqlite::params;

use crate::error::ApiError;
use crate::state::AppState;

const DEFAULT_NODE_LIMIT: usize = 500;
const MAX_NODE_LIMIT: usize = 2000;

#[derive(Deserialize)]
pub struct GraphQuery {
    pub community_id: Option<i64>,
    pub limit: Option<usize>,
    pub offset: Option<usize>,
}

#[derive(Serialize)]
pub struct GraphResponse {
    pub nodes: Vec<GraphNodeResponse>,
    pub edges: Vec<GraphEdgeResponse>,
}

#[derive(Serialize)]
pub struct GraphNodeResponse {
    pub id: i64,
    pub name: String,
    pub kind: String,
    pub file_id: i64,
    pub community_id: Option<i64>,
}

#[derive(Serialize)]
pub struct GraphEdgeResponse {
    pub source_id: i64,
    pub target_id: i64,
    pub kind: String,
    pub confidence: f64,
}

#[derive(Serialize)]
pub struct CommunityResponse {
    pub id: i64,
    pub label: String,
    pub cohesion: f64,
    pub member_count: usize,
    pub members: Vec<i64>,
}

#[derive(Serialize)]
pub struct ProcessResponse {
    pub id: i64,
    pub name: String,
    pub entry_symbol_id: i64,
    pub steps: Vec<ProcessStepResponse>,
}

#[derive(Serialize)]
pub struct ProcessStepResponse {
    pub order: i64,
    pub symbol_id: i64,
    pub symbol_name: String,
    pub description: String,
}

/// GET /api/v1/graph — Paginated graph (nodes + edges)
///
/// Query parameters:
/// - `community_id` — filter to nodes in this community
/// - `limit` — max nodes to return (default 500, max 2000)
/// - `offset` — pagination offset (default 0)
async fn get_graph(
    State(state): State<AppState>,
    Query(query): Query<GraphQuery>,
) -> Result<Json<GraphResponse>, ApiError> {
    let conn = state.db.connection();

    let limit = query.limit.unwrap_or(DEFAULT_NODE_LIMIT).min(MAX_NODE_LIMIT);
    let offset = query.offset.unwrap_or(0);

    // Load symbols as graph nodes, with optional community filter and pagination
    let nodes = if let Some(cid) = query.community_id {
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.name, s.kind, s.file_id, cm.community_id \
                 FROM symbols s \
                 INNER JOIN community_members cm ON cm.symbol_id = s.id \
                 WHERE cm.community_id = ?1 \
                 LIMIT ?2 OFFSET ?3",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![cid, limit as i64, offset as i64], |row| {
                Ok(GraphNodeResponse {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    file_id: row.get(3)?,
                    community_id: row.get(4)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?
    } else {
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.name, s.kind, s.file_id, cm.community_id \
                 FROM symbols s \
                 LEFT JOIN community_members cm ON cm.symbol_id = s.id \
                 LIMIT ?1 OFFSET ?2",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map(params![limit as i64, offset as i64], |row| {
                Ok(GraphNodeResponse {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    kind: row.get(2)?,
                    file_id: row.get(3)?,
                    community_id: row.get(4)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        rows.collect::<Result<Vec<_>, _>>()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?
    };

    // Build a set of returned node IDs so we only return edges within the node set
    let node_ids: HashSet<i64> = nodes.iter().map(|n| n.id).collect();

    // Load edges, filtering to only those between returned nodes
    let mut edges = Vec::new();
    {
        let mut stmt = conn
            .prepare("SELECT source_id, target_id, kind, confidence FROM edges")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let rows = stmt
            .query_map([], |row| {
                Ok(GraphEdgeResponse {
                    source_id: row.get(0)?,
                    target_id: row.get(1)?,
                    kind: row.get(2)?,
                    confidence: row.get(3)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        for row in rows {
            let edge = row.map_err(|e| CodeilusError::Database(Box::new(e)))?;
            if node_ids.contains(&edge.source_id) && node_ids.contains(&edge.target_id) {
                edges.push(edge);
            }
        }
    }

    Ok(Json(GraphResponse { nodes, edges }))
}

/// GET /api/v1/communities — List all communities
async fn list_communities(
    State(state): State<AppState>,
) -> Result<Json<Vec<CommunityResponse>>, ApiError> {
    let conn = state.db.connection();
    let mut communities = Vec::new();

    let mut stmt = conn
        .prepare("SELECT id, name, cohesion_score FROM communities")
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<f64>>(2)?,
            ))
        })
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;

    for row in rows {
        let (id, name, cohesion) =
            row.map_err(|e| CodeilusError::Database(Box::new(e)))?;

        // Fetch members for this community
        let mut member_stmt = conn
            .prepare("SELECT symbol_id FROM community_members WHERE community_id = ?1")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let members: Vec<i64> = member_stmt
            .query_map(params![id], |row| row.get(0))
            .map_err(|e| CodeilusError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        communities.push(CommunityResponse {
            id,
            label: name.unwrap_or_default(),
            cohesion: cohesion.unwrap_or(0.0),
            member_count: members.len(),
            members,
        });
    }

    Ok(Json(communities))
}

/// GET /api/v1/processes — List all execution flows
async fn list_processes(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProcessResponse>>, ApiError> {
    let conn = state.db.connection();
    let mut processes = Vec::new();

    let mut stmt = conn
        .prepare("SELECT id, name, entry_symbol_id FROM processes")
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
    let rows = stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<i64>>(2)?,
            ))
        })
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;

    for row in rows {
        let (id, name, entry_symbol_id) =
            row.map_err(|e| CodeilusError::Database(Box::new(e)))?;

        // Fetch steps with symbol names
        let mut step_stmt = conn
            .prepare(
                "SELECT ps.step_order, ps.symbol_id, COALESCE(s.name, ''), COALESCE(p.description, '') \
                 FROM process_steps ps \
                 LEFT JOIN symbols s ON s.id = ps.symbol_id \
                 LEFT JOIN processes p ON p.id = ps.process_id \
                 WHERE ps.process_id = ?1 \
                 ORDER BY ps.step_order",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let steps: Vec<ProcessStepResponse> = step_stmt
            .query_map(params![id], |row| {
                Ok(ProcessStepResponse {
                    order: row.get(0)?,
                    symbol_id: row.get(1)?,
                    symbol_name: row.get(2)?,
                    description: row.get(3)?,
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        processes.push(ProcessResponse {
            id,
            name: name.unwrap_or_default(),
            entry_symbol_id: entry_symbol_id.unwrap_or(0),
            steps,
        });
    }

    Ok(Json(processes))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/graph", get(get_graph))
        .route("/communities", get(list_communities))
        .route("/processes", get(list_processes))
}
