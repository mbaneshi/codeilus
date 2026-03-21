//! Graph, community, and process API routes.

use std::collections::{HashMap, HashSet};

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

#[derive(Serialize)]
pub struct CommunityGraphResponse {
    pub nodes: Vec<CommunityGraphNode>,
    pub edges: Vec<CommunityGraphEdge>,
}

#[derive(Serialize)]
pub struct CommunityGraphNode {
    pub id: i64,
    pub label: String,
    pub member_count: usize,
    pub cohesion: f64,
}

#[derive(Serialize)]
pub struct CommunityGraphEdge {
    pub source_id: i64,
    pub target_id: i64,
    pub weight: usize,
}

/// GET /api/v1/graph/communities — Community-level graph
///
/// Returns communities as nodes and aggregated inter-community edges.
async fn get_community_graph(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, ApiError> {
    let cache_key = "graph:communities".to_string();
    if let Some(cached) = state.cache.json.get(&cache_key) {
        return Ok(Json(cached));
    }

    let conn = state.db.connection();

    // 1. Load all communities
    let mut comm_stmt = conn
        .prepare("SELECT id, name, cohesion_score FROM communities")
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
    let comm_rows = comm_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<f64>>(2)?,
            ))
        })
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;

    let mut community_info: HashMap<i64, (String, f64)> = HashMap::new();
    for row in comm_rows {
        let (id, name, cohesion) = row.map_err(|e| CodeilusError::Database(Box::new(e)))?;
        community_info.insert(id, (name.unwrap_or_default(), cohesion.unwrap_or(0.0)));
    }

    // 2. Load symbol -> community mappings
    let mut member_stmt = conn
        .prepare("SELECT symbol_id, community_id FROM community_members")
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
    let member_rows = member_stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;

    let mut symbol_to_community: HashMap<i64, i64> = HashMap::new();
    let mut community_member_counts: HashMap<i64, usize> = HashMap::new();
    for row in member_rows {
        let (symbol_id, community_id) =
            row.map_err(|e| CodeilusError::Database(Box::new(e)))?;
        symbol_to_community.insert(symbol_id, community_id);
        *community_member_counts.entry(community_id).or_insert(0) += 1;
    }

    // 3. Build community nodes
    let nodes: Vec<CommunityGraphNode> = community_info
        .iter()
        .map(|(&id, (label, cohesion))| CommunityGraphNode {
            id,
            label: label.clone(),
            member_count: community_member_counts.get(&id).copied().unwrap_or(0),
            cohesion: *cohesion,
        })
        .collect();

    // 4. Load all edges and aggregate to community level
    let mut edge_stmt = conn
        .prepare("SELECT source_id, target_id FROM edges")
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
    let edge_rows = edge_stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;

    let mut inter_community_edges: HashMap<(i64, i64), usize> = HashMap::new();
    for row in edge_rows {
        let (src, dst) = row.map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let src_comm = symbol_to_community.get(&src);
        let dst_comm = symbol_to_community.get(&dst);
        if let (Some(&sc), Some(&dc)) = (src_comm, dst_comm) {
            if sc != dc {
                *inter_community_edges.entry((sc, dc)).or_insert(0) += 1;
            }
        }
    }

    let edges: Vec<CommunityGraphEdge> = inter_community_edges
        .into_iter()
        .map(|((source_id, target_id), weight)| CommunityGraphEdge {
            source_id,
            target_id,
            weight,
        })
        .collect();

    let response = CommunityGraphResponse { nodes, edges };
    let value = serde_json::to_value(&response)
        .map_err(|e| ApiError::from(CodeilusError::Internal(e.to_string())))?;
    state.cache.json.insert(cache_key, value.clone());

    Ok(Json(value))
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
) -> Result<Json<serde_json::Value>, ApiError> {
    let limit = query.limit.unwrap_or(DEFAULT_NODE_LIMIT).min(MAX_NODE_LIMIT);
    let offset = query.offset.unwrap_or(0);

    let cache_key = format!(
        "graph:c={:?}:l={}:o={}",
        query.community_id, limit, offset
    );
    if let Some(cached) = state.cache.json.get(&cache_key) {
        return Ok(Json(cached));
    }

    let conn = state.db.connection();

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

    let response = serde_json::to_value(GraphResponse { nodes, edges })
        .map_err(|e| ApiError::from(CodeilusError::Internal(e.to_string())))?;
    state.cache.json.insert(cache_key, response.clone());
    Ok(Json(response))
}

/// GET /api/v1/communities — List all communities
async fn list_communities(
    State(state): State<AppState>,
) -> Result<Json<Vec<CommunityResponse>>, ApiError> {
    let conn = state.db.connection();

    // Load all communities
    let mut comm_stmt = conn
        .prepare("SELECT id, name, cohesion_score FROM communities")
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
    let comm_rows = comm_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<f64>>(2)?,
            ))
        })
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;

    let mut community_info: Vec<(i64, String, f64)> = Vec::new();
    for row in comm_rows {
        let (id, name, cohesion) = row.map_err(|e| CodeilusError::Database(Box::new(e)))?;
        community_info.push((id, name.unwrap_or_default(), cohesion.unwrap_or(0.0)));
    }

    // Batch-load all members in one query, group by community_id
    let mut member_stmt = conn
        .prepare("SELECT community_id, symbol_id FROM community_members")
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
    let member_rows = member_stmt
        .query_map([], |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)))
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;

    let mut members_by_community: HashMap<i64, Vec<i64>> = HashMap::new();
    for row in member_rows {
        let (community_id, symbol_id) = row.map_err(|e| CodeilusError::Database(Box::new(e)))?;
        members_by_community
            .entry(community_id)
            .or_default()
            .push(symbol_id);
    }

    let communities: Vec<CommunityResponse> = community_info
        .into_iter()
        .map(|(id, label, cohesion)| {
            let members = members_by_community.remove(&id).unwrap_or_default();
            CommunityResponse {
                id,
                label,
                cohesion,
                member_count: members.len(),
                members,
            }
        })
        .collect();

    Ok(Json(communities))
}

/// GET /api/v1/processes — List all execution flows
async fn list_processes(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProcessResponse>>, ApiError> {
    let conn = state.db.connection();

    // Load all processes
    let mut proc_stmt = conn
        .prepare("SELECT id, name, entry_symbol_id FROM processes")
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
    let proc_rows = proc_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, Option<String>>(1)?,
                row.get::<_, Option<i64>>(2)?,
            ))
        })
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;

    let mut process_info: Vec<(i64, String, i64)> = Vec::new();
    for row in proc_rows {
        let (id, name, entry_symbol_id) = row.map_err(|e| CodeilusError::Database(Box::new(e)))?;
        process_info.push((id, name.unwrap_or_default(), entry_symbol_id.unwrap_or(0)));
    }

    // Batch-load all steps with symbol names in one query
    let mut step_stmt = conn
        .prepare(
            "SELECT ps.process_id, ps.step_order, ps.symbol_id, COALESCE(s.name, ''), COALESCE(p.description, '') \
             FROM process_steps ps \
             LEFT JOIN symbols s ON s.id = ps.symbol_id \
             LEFT JOIN processes p ON p.id = ps.process_id \
             ORDER BY ps.process_id, ps.step_order",
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
    let step_rows = step_stmt
        .query_map([], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                ProcessStepResponse {
                    order: row.get(1)?,
                    symbol_id: row.get(2)?,
                    symbol_name: row.get(3)?,
                    description: row.get(4)?,
                },
            ))
        })
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;

    let mut steps_by_process: HashMap<i64, Vec<ProcessStepResponse>> = HashMap::new();
    for row in step_rows {
        let (process_id, step) = row.map_err(|e| CodeilusError::Database(Box::new(e)))?;
        steps_by_process.entry(process_id).or_default().push(step);
    }

    let processes: Vec<ProcessResponse> = process_info
        .into_iter()
        .map(|(id, name, entry_symbol_id)| {
            let steps = steps_by_process.remove(&id).unwrap_or_default();
            ProcessResponse {
                id,
                name,
                entry_symbol_id,
                steps,
            }
        })
        .collect();

    Ok(Json(processes))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/graph", get(get_graph))
        .route("/graph/communities", get(get_community_graph))
        .route("/communities", get(list_communities))
        .route("/processes", get(list_processes))
}
