//! Integration tests: parse → store → graph → API round-trip.

use std::path::PathBuf;
use std::sync::Arc;

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use codeilus_api::{app, AppState};
use codeilus_core::EventBus;
use codeilus_db::{CommunityRepo, DbPool, EdgeRepo, FileRepo, Migrator, SymbolRepo};
use codeilus_graph::GraphBuilder;
use codeilus_parse::{ParseConfig, parse_repository};

/// Parse codeilus-core's src/ as a fixture (small, fast, always present).
fn parse_fixture() -> Vec<codeilus_parse::ParsedFile> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../codeilus-core/src");
    let config = ParseConfig::new(root);
    parse_repository(&config, None).expect("parse should succeed")
}

fn setup_db() -> Arc<DbPool> {
    let db = DbPool::in_memory().unwrap();
    {
        let conn = db.connection();
        Migrator::new(&conn).apply_pending().unwrap();
    }
    Arc::new(db)
}

fn setup_state(db: Arc<DbPool>) -> AppState {
    let event_bus = Arc::new(EventBus::new(16));
    AppState::new(db, event_bus)
}

/// Store parsed files and symbols into the DB; returns the DB pool.
fn store_parsed(parsed: &[codeilus_parse::ParsedFile]) -> Arc<DbPool> {
    let db = setup_db();
    let conn = db.conn_arc();
    let file_repo = FileRepo::new(Arc::clone(&conn));
    let sym_repo = SymbolRepo::new(Arc::clone(&conn));

    for pf in parsed {
        let path_str = pf.path.to_string_lossy();
        let lang_str = pf.language.as_str();
        let file_id = file_repo
            .insert(&path_str, Some(lang_str), pf.sloc as i64)
            .expect("file insert");

        for sym in &pf.symbols {
            let kind_str = format!("{:?}", sym.kind); // Debug gives PascalCase
            sym_repo
                .insert(
                    file_id,
                    &sym.name,
                    &kind_str,
                    sym.start_line,
                    sym.end_line,
                    sym.signature.as_deref(),
                )
                .expect("symbol insert");
        }
    }
    db
}

async fn get_json(state: &AppState, uri: &str) -> (StatusCode, serde_json::Value) {
    let app = app(state.clone());
    let req = Request::builder()
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    (status, json)
}

// ── Parse + Store round-trip ─────────────────────────────────

#[test]
fn parse_fixture_produces_files_and_symbols() {
    let parsed = parse_fixture();
    assert!(!parsed.is_empty(), "should parse at least one file");
    let total_symbols: usize = parsed.iter().map(|pf| pf.symbols.len()).sum();
    assert!(total_symbols > 0, "should extract symbols");
}

#[test]
fn store_parsed_data_in_db() {
    let parsed = parse_fixture();
    let db = store_parsed(&parsed);
    let conn = db.conn_arc();
    let file_repo = FileRepo::new(Arc::clone(&conn));
    let sym_repo = SymbolRepo::new(conn);

    assert!(file_repo.count().unwrap() > 0, "files stored");
    assert!(sym_repo.count().unwrap() > 0, "symbols stored");
}

// ── Graph building ───────────────────────────────────────────

#[test]
fn graph_builder_produces_communities() {
    let parsed = parse_fixture();
    let graph = GraphBuilder::new()
        .build(&parsed)
        .expect("graph build should succeed");

    // petgraph edges
    let edge_count = graph.graph.edge_count();
    assert!(
        edge_count > 0,
        "graph should have edges (got {})",
        edge_count
    );
    assert!(
        !graph.communities.is_empty(),
        "graph should detect communities (got {})",
        graph.communities.len()
    );
}

// ── API round-trip: graph endpoint ───────────────────────────

#[tokio::test]
async fn graph_api_returns_nodes_with_community_ids() {
    let parsed = parse_fixture();
    let db = store_parsed(&parsed);

    // Build graph and store edges + communities
    let graph = GraphBuilder::new().build(&parsed).unwrap();
    let conn = db.conn_arc();

    let edge_repo = EdgeRepo::new(Arc::clone(&conn));
    for edge_idx in graph.graph.edge_indices() {
        let (src, tgt) = graph.graph.edge_endpoints(edge_idx).unwrap();
        let src_node = &graph.graph[src];
        let tgt_node = &graph.graph[tgt];
        let edge_data = &graph.graph[edge_idx];
        edge_repo
            .insert(
                src_node.symbol_id,
                tgt_node.symbol_id,
                &format!("{:?}", edge_data.kind),
                edge_data.confidence.0,
            )
            .unwrap();
    }

    let comm_repo = CommunityRepo::new(Arc::clone(&conn));
    for c in &graph.communities {
        let cid = comm_repo.insert(&c.label, c.cohesion).unwrap();
        let members: Vec<_> = c.members.iter().map(|s| (cid, *s)).collect();
        if !members.is_empty() {
            comm_repo.insert_members_batch(&members).unwrap();
        }
    }

    // Test API
    let state = setup_state(db);
    let (status, body) = get_json(&state, "/api/v1/graph").await;
    assert_eq!(status, StatusCode::OK);

    let nodes = body["nodes"].as_array().unwrap();
    let edges = body["edges"].as_array().unwrap();
    assert!(!nodes.is_empty(), "graph API should return nodes");
    assert!(!edges.is_empty(), "graph API should return edges");

    // Verify nodes have community_id
    let with_community = nodes
        .iter()
        .filter(|n| !n["community_id"].is_null())
        .count();
    assert!(
        with_community > 0,
        "some nodes should have community_id assigned"
    );
}

// ── API round-trip: files endpoint ───────────────────────────

#[tokio::test]
async fn files_api_returns_stored_files() {
    let parsed = parse_fixture();
    let db = store_parsed(&parsed);
    let state = setup_state(db);

    let (status, body) = get_json(&state, "/api/v1/files").await;
    assert_eq!(status, StatusCode::OK);
    let files = body.as_array().unwrap();
    assert_eq!(
        files.len(),
        parsed.len(),
        "API should return all parsed files"
    );
}

// ── API round-trip: communities endpoint ─────────────────────

#[tokio::test]
async fn communities_api_returns_stored_communities() {
    let parsed = parse_fixture();
    let db = store_parsed(&parsed); // need symbols for FK on community_members

    let graph = GraphBuilder::new().build(&parsed).unwrap();
    let comm_repo = CommunityRepo::new(db.conn_arc());
    for c in &graph.communities {
        let cid = comm_repo.insert(&c.label, c.cohesion).unwrap();
        let members: Vec<_> = c.members.iter().map(|s| (cid, *s)).collect();
        if !members.is_empty() {
            comm_repo.insert_members_batch(&members).unwrap();
        }
    }

    let state = setup_state(db);
    let (status, body) = get_json(&state, "/api/v1/communities").await;
    assert_eq!(status, StatusCode::OK);
    let communities = body.as_array().unwrap();
    assert!(
        !communities.is_empty(),
        "should return communities"
    );
    assert!(
        communities.len() <= 50,
        "community count should be reasonable (got {})",
        communities.len()
    );
}

// ── API round-trip: source endpoint ──────────────────────────

#[tokio::test]
async fn source_endpoint_with_repo_root() {
    let db = setup_db();
    let file_repo = FileRepo::new(db.conn_arc());

    // Insert a file that we know exists on disk
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let repo_root = manifest_dir.join("../codeilus-core");
    let file_id = file_repo
        .insert("src/lib.rs", Some("rust"), 10)
        .unwrap();

    let event_bus = Arc::new(EventBus::new(16));
    let state = AppState::new(db, event_bus).with_repo_root(repo_root);

    let (status, body) = get_json(
        &state,
        &format!("/api/v1/files/{}/source?start=1&end=5", file_id.0),
    )
    .await;
    assert_eq!(status, StatusCode::OK);

    let lines = body["lines"].as_array().unwrap();
    assert_eq!(lines.len(), 5, "should return 5 source lines");
    assert_eq!(lines[0]["number"], 1);
    assert!(
        !lines[0]["content"].as_str().unwrap().is_empty(),
        "source lines should have content"
    );
}
