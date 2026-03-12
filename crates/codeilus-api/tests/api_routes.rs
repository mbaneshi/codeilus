use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use codeilus_api::{app, AppState};
use codeilus_core::EventBus;
use codeilus_db::{DbPool, FileRepo, Migrator, SymbolRepo};
use std::sync::Arc;

fn setup() -> AppState {
    let db = DbPool::in_memory().unwrap();
    {
        let conn = db.connection();
        Migrator::new(&conn).apply_pending().unwrap();
    }
    let db = Arc::new(db);
    let event_bus = Arc::new(EventBus::new(16));
    AppState::new(db, event_bus)
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

// ── Health ────────────────────────────────────────────────────

#[tokio::test]
async fn health_returns_ok() {
    let state = setup();
    let (status, body) = get_json(&state, "/api/v1/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
}

// ── Files ─────────────────────────────────────────────────────

#[tokio::test]
async fn list_files_empty() {
    let state = setup();
    let (status, body) = get_json(&state, "/api/v1/files").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!([]));
}

#[tokio::test]
async fn list_files_with_data() {
    let state = setup();
    let repo = FileRepo::new(state.db.conn_arc());
    repo.insert("a.rs", Some("rust"), 10).unwrap();
    repo.insert("b.py", Some("python"), 20).unwrap();
    repo.insert("c.go", Some("go"), 30).unwrap();

    let (status, body) = get_json(&state, "/api/v1/files").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 3);
}

#[tokio::test]
async fn list_files_language_filter() {
    let state = setup();
    let repo = FileRepo::new(state.db.conn_arc());
    repo.insert("a.py", Some("python"), 10).unwrap();
    repo.insert("b.rs", Some("rust"), 20).unwrap();
    repo.insert("c.py", Some("python"), 30).unwrap();

    let (status, body) = get_json(&state, "/api/v1/files?language=python").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 2);
    for item in arr {
        assert_eq!(item["language"], "python");
    }
}

#[tokio::test]
async fn get_file_by_id() {
    let state = setup();
    let repo = FileRepo::new(state.db.conn_arc());
    let id = repo.insert("src/main.rs", Some("rust"), 100).unwrap();

    let (status, body) = get_json(&state, &format!("/api/v1/files/{}", id.0)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["path"], "src/main.rs");
    assert_eq!(body["language"], "rust");
}

#[tokio::test]
async fn get_file_not_found() {
    let state = setup();
    let (status, body) = get_json(&state, "/api/v1/files/999").await;
    assert_eq!(status, StatusCode::NOT_FOUND);
    assert!(body["error"].as_str().unwrap().contains("Not found"));
}

#[tokio::test]
async fn get_file_symbols() {
    let state = setup();
    let file_repo = FileRepo::new(state.db.conn_arc());
    let file_id = file_repo.insert("lib.rs", Some("rust"), 50).unwrap();

    let sym_repo = SymbolRepo::new(state.db.conn_arc());
    sym_repo
        .insert(file_id, "main", "Function", 1, 10, Some("fn main()"))
        .unwrap();
    sym_repo
        .insert(file_id, "helper", "Function", 11, 20, None)
        .unwrap();

    let (status, body) = get_json(&state, &format!("/api/v1/files/{}/symbols", file_id.0)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

// ── Symbols ───────────────────────────────────────────────────

#[tokio::test]
async fn list_symbols() {
    let state = setup();
    let file_repo = FileRepo::new(state.db.conn_arc());
    let fid = file_repo.insert("a.rs", Some("rust"), 10).unwrap();
    let sym_repo = SymbolRepo::new(state.db.conn_arc());
    sym_repo.insert(fid, "foo", "Function", 1, 5, None).unwrap();
    sym_repo.insert(fid, "Bar", "Class", 6, 20, None).unwrap();

    let (status, body) = get_json(&state, "/api/v1/symbols").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn get_symbol_by_id() {
    let state = setup();
    let file_repo = FileRepo::new(state.db.conn_arc());
    let fid = file_repo.insert("a.rs", Some("rust"), 10).unwrap();
    let sym_repo = SymbolRepo::new(state.db.conn_arc());
    let sid = sym_repo
        .insert(fid, "process", "Function", 1, 10, Some("fn process()"))
        .unwrap();

    let (status, body) = get_json(&state, &format!("/api/v1/symbols/{}", sid.0)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["name"], "process");
    assert_eq!(body["kind"], "Function");
}

#[tokio::test]
async fn search_symbols_prefix() {
    let state = setup();
    let file_repo = FileRepo::new(state.db.conn_arc());
    let fid = file_repo.insert("a.rs", Some("rust"), 10).unwrap();
    let sym_repo = SymbolRepo::new(state.db.conn_arc());
    sym_repo.insert(fid, "process", "Function", 1, 5, None).unwrap();
    sym_repo.insert(fid, "processor", "Class", 6, 20, None).unwrap();
    sym_repo.insert(fid, "handle", "Function", 21, 30, None).unwrap();

    let (status, body) = get_json(&state, "/api/v1/symbols/search?q=proc").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

// ── Graph ─────────────────────────────────────────────────────

#[tokio::test]
async fn get_graph_empty() {
    let state = setup();
    let (status, body) = get_json(&state, "/api/v1/graph").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["nodes"], serde_json::json!([]));
    assert_eq!(body["edges"], serde_json::json!([]));
}

#[tokio::test]
async fn list_communities_empty() {
    let state = setup();
    let (status, body) = get_json(&state, "/api/v1/communities").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!([]));
}

#[tokio::test]
async fn list_processes_empty() {
    let state = setup();
    let (status, body) = get_json(&state, "/api/v1/processes").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!([]));
}
