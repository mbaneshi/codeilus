use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

use codeilus_api::{app, AppState};
use codeilus_core::{CodeilusConfig, EventBus};
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
    let llm: Arc<dyn codeilus_llm::LlmProvider> = Arc::new(codeilus_llm::ClaudeCli::new());
    let config = Arc::new(CodeilusConfig::default());
    AppState::new(db, event_bus, llm, config)
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

async fn post_json(
    state: &AppState,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let app = app(state.clone());
    let req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn put_json(
    state: &AppState,
    uri: &str,
    body: serde_json::Value,
) -> (StatusCode, serde_json::Value) {
    let app = app(state.clone());
    let req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("Content-Type", "application/json")
        .body(Body::from(serde_json::to_vec(&body).unwrap()))
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
    (status, json)
}

async fn delete_request(
    state: &AppState,
    uri: &str,
) -> (StatusCode, serde_json::Value) {
    let app = app(state.clone());
    let req = Request::builder()
        .method("DELETE")
        .uri(uri)
        .body(Body::empty())
        .unwrap();
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let body = axum::body::to_bytes(resp.into_body(), usize::MAX)
        .await
        .unwrap();
    let json: serde_json::Value =
        serde_json::from_slice(&body).unwrap_or(serde_json::Value::Null);
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
    let repo = FileRepo::new(Arc::clone(&state.db));
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
    let repo = FileRepo::new(Arc::clone(&state.db));
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
    let repo = FileRepo::new(Arc::clone(&state.db));
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
    let file_repo = FileRepo::new(Arc::clone(&state.db));
    let file_id = file_repo.insert("lib.rs", Some("rust"), 50).unwrap();

    let sym_repo = SymbolRepo::new(Arc::clone(&state.db));
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
    let file_repo = FileRepo::new(Arc::clone(&state.db));
    let fid = file_repo.insert("a.rs", Some("rust"), 10).unwrap();
    let sym_repo = SymbolRepo::new(Arc::clone(&state.db));
    sym_repo.insert(fid, "foo", "Function", 1, 5, None).unwrap();
    sym_repo.insert(fid, "Bar", "Class", 6, 20, None).unwrap();

    let (status, body) = get_json(&state, "/api/v1/symbols").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body.as_array().unwrap().len(), 2);
}

#[tokio::test]
async fn get_symbol_by_id() {
    let state = setup();
    let file_repo = FileRepo::new(Arc::clone(&state.db));
    let fid = file_repo.insert("a.rs", Some("rust"), 10).unwrap();
    let sym_repo = SymbolRepo::new(Arc::clone(&state.db));
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
    let file_repo = FileRepo::new(Arc::clone(&state.db));
    let fid = file_repo.insert("a.rs", Some("rust"), 10).unwrap();
    let sym_repo = SymbolRepo::new(Arc::clone(&state.db));
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

// ── Health (enhanced) ────────────────────────────────────────

#[tokio::test]
async fn health_returns_enhanced_fields() {
    let state = setup();
    // Insert some data so the counts are non-zero
    let file_repo = FileRepo::new(Arc::clone(&state.db));
    let fid = file_repo.insert("x.rs", Some("rust"), 10).unwrap();
    let sym_repo = SymbolRepo::new(Arc::clone(&state.db));
    sym_repo.insert(fid, "main", "Function", 1, 5, None).unwrap();
    sym_repo.insert(fid, "helper", "Function", 6, 10, None).unwrap();

    let (status, body) = get_json(&state, "/api/v1/health").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "ok");
    assert!(body["version"].is_string());
    assert_eq!(body["db_ok"], true);
    // llm_available may be true or false depending on environment
    assert!(body["llm_available"].is_boolean());
    assert_eq!(body["files_analyzed"], 1);
    assert_eq!(body["symbols_count"], 2);
}

// ── Annotations CRUD ─────────────────────────────────────────

#[tokio::test]
async fn annotation_create_and_list() {
    let state = setup();

    // Create an annotation (target_type = "file", target_id = 1 — no FK constraint)
    let (status, body) = post_json(
        &state,
        "/api/v1/annotations",
        serde_json::json!({
            "target_type": "file",
            "target_id": 1,
            "content": "test note"
        }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["content"], "test note");
    assert_eq!(body["target_type"], "file");
    assert_eq!(body["target_id"], 1);
    assert!(body["id"].is_number());

    // List all annotations
    let (status, body) = get_json(&state, "/api/v1/annotations").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["content"], "test note");
}

#[tokio::test]
async fn annotation_list_by_target() {
    let state = setup();

    // Create two annotations on different targets
    post_json(
        &state,
        "/api/v1/annotations",
        serde_json::json!({
            "target_type": "file",
            "target_id": 1,
            "content": "note on file 1"
        }),
    )
    .await;
    post_json(
        &state,
        "/api/v1/annotations",
        serde_json::json!({
            "target_type": "symbol",
            "target_id": 42,
            "content": "note on symbol 42"
        }),
    )
    .await;

    // List annotations for file/1
    let (status, body) = get_json(&state, "/api/v1/annotations/file/1").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["content"], "note on file 1");

    // List annotations for symbol/42
    let (status, body) = get_json(&state, "/api/v1/annotations/symbol/42").await;
    assert_eq!(status, StatusCode::OK);
    let arr = body.as_array().unwrap();
    assert_eq!(arr.len(), 1);
    assert_eq!(arr[0]["content"], "note on symbol 42");
}

#[tokio::test]
async fn annotation_update() {
    let state = setup();

    // Create
    let (_, body) = post_json(
        &state,
        "/api/v1/annotations",
        serde_json::json!({
            "target_type": "file",
            "target_id": 1,
            "content": "original"
        }),
    )
    .await;
    let id = body["id"].as_i64().unwrap();

    // Update
    let (status, body) = put_json(
        &state,
        &format!("/api/v1/annotations/{}", id),
        serde_json::json!({ "content": "updated" }),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["updated"], true);

    // Verify via list
    let (_, body) = get_json(&state, "/api/v1/annotations").await;
    assert_eq!(body.as_array().unwrap()[0]["content"], "updated");
}

#[tokio::test]
async fn annotation_delete() {
    let state = setup();

    // Create
    let (_, body) = post_json(
        &state,
        "/api/v1/annotations",
        serde_json::json!({
            "target_type": "file",
            "target_id": 1,
            "content": "to delete"
        }),
    )
    .await;
    let id = body["id"].as_i64().unwrap();

    // Delete
    let (status, body) = delete_request(&state, &format!("/api/v1/annotations/{}", id)).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["deleted"], true);

    // Verify gone
    let (_, body) = get_json(&state, "/api/v1/annotations").await;
    assert_eq!(body.as_array().unwrap().len(), 0);
}

#[tokio::test]
async fn annotation_toggle_flag() {
    let state = setup();

    // Create
    let (_, body) = post_json(
        &state,
        "/api/v1/annotations",
        serde_json::json!({
            "target_type": "file",
            "target_id": 1,
            "content": "flaggable"
        }),
    )
    .await;
    let id = body["id"].as_i64().unwrap();
    assert_eq!(body["flagged"], false);

    // Toggle flag on
    let (status, body) = post_json(
        &state,
        &format!("/api/v1/annotations/{}/flag", id),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["flagged"], true);

    // Toggle flag off
    let (status, body) = post_json(
        &state,
        &format!("/api/v1/annotations/{}/flag", id),
        serde_json::json!({}),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["flagged"], false);
}

// ── Learning endpoints ───────────────────────────────────────

#[tokio::test]
async fn progress_empty() {
    let state = setup();
    let (status, body) = get_json(&state, "/api/v1/progress").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body, serde_json::json!([]));
}

#[tokio::test]
async fn learner_stats_initial() {
    let state = setup();
    let (status, body) = get_json(&state, "/api/v1/learner/stats").await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["total_xp"], 0);
    assert_eq!(body["streak_days"], 0);
    assert_eq!(body["chapters_completed"], 0);
    assert!(body["badges"].as_array().unwrap().is_empty());
}

// ── Graph pagination ─────────────────────────────────────────

#[tokio::test]
async fn graph_pagination() {
    let state = setup();
    let file_repo = FileRepo::new(Arc::clone(&state.db));
    let fid = file_repo.insert("a.rs", Some("rust"), 50).unwrap();
    let sym_repo = SymbolRepo::new(Arc::clone(&state.db));
    sym_repo.insert(fid, "alpha", "Function", 1, 5, None).unwrap();
    sym_repo.insert(fid, "beta", "Function", 6, 10, None).unwrap();
    sym_repo.insert(fid, "gamma", "Function", 11, 15, None).unwrap();
    sym_repo.insert(fid, "delta", "Function", 16, 20, None).unwrap();

    // Request limit=2, offset=0
    let (status, body) = get_json(&state, "/api/v1/graph?limit=2&offset=0").await;
    assert_eq!(status, StatusCode::OK);
    let nodes = body["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 2);

    // Request limit=2, offset=2 (second page)
    let (status, body) = get_json(&state, "/api/v1/graph?limit=2&offset=2").await;
    assert_eq!(status, StatusCode::OK);
    let nodes = body["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 2);

    // Request limit=2, offset=4 (past end)
    let (status, body) = get_json(&state, "/api/v1/graph?limit=2&offset=4").await;
    assert_eq!(status, StatusCode::OK);
    let nodes = body["nodes"].as_array().unwrap();
    assert_eq!(nodes.len(), 0);
}

// ── Search edge cases ────────────────────────────────────────

#[tokio::test]
async fn search_empty_query() {
    let state = setup();
    let (status, body) = get_json(&state, "/api/v1/search?q=").await;
    assert_eq!(status, StatusCode::OK);
    assert!(body["results"].is_array());
}

#[tokio::test]
async fn search_nonexistent_term() {
    let state = setup();
    // Insert some data so there's something to search through
    let file_repo = FileRepo::new(Arc::clone(&state.db));
    let fid = file_repo.insert("a.rs", Some("rust"), 10).unwrap();
    let sym_repo = SymbolRepo::new(Arc::clone(&state.db));
    sym_repo.insert(fid, "real_function", "Function", 1, 5, None).unwrap();

    let (status, body) = get_json(&state, "/api/v1/search?q=nonexistent_xyz").await;
    assert_eq!(status, StatusCode::OK);
    let results = body["results"].as_array().unwrap();
    assert_eq!(results.len(), 0);
}
