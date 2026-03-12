# Task: API Routes (Files, Symbols, Graph)

> **Crate:** `crates/codeilus-api/`
> **Wave:** 2 (parallel with graph)
> **Depends on:** codeilus-core (done), codeilus-db repos (wave 1)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 9, Sprints 1-2 for API deliverables
- `crates/codeilus-api/src/lib.rs` — existing Axum server setup
- `crates/codeilus-api/src/routes/health.rs` — example route pattern
- `crates/codeilus-api/src/routes/ws.rs` — WebSocket handler pattern
- `crates/codeilus-db/src/repos/file_repo.rs` — FileRepo API
- `crates/codeilus-db/src/repos/symbol_repo.rs` — SymbolRepo API
- `crates/codeilus-db/src/repos/edge_repo.rs` — EdgeRepo API
- `crates/codeilus-db/src/repos/community_repo.rs` — CommunityRepo API (wave 2 graph creates this)
- `crates/codeilus-db/src/repos/process_repo.rs` — ProcessRepo API (wave 2 graph creates this)

## Objective

Add REST API routes for files, symbols, and graph data. All routes return JSON. Use Axum extractors with shared DB state. Follow the existing health route pattern.

Base path: `/api/v1/`

## Files to Create/Modify

### 1. `src/routes/files.rs` — File routes

```rust
use axum::{extract::{Path, State, Query}, Json, Router};
use axum::routing::get;
use serde::{Serialize, Deserialize};

#[derive(Deserialize)]
pub struct FileListQuery {
    pub language: Option<String>,
}

/// GET /api/v1/files — List all files, optional ?language= filter
pub async fn list_files(
    State(state): State<AppState>,
    Query(query): Query<FileListQuery>,
) -> Result<Json<Vec<FileRow>>, ApiError> { ... }

/// GET /api/v1/files/:id — Get a single file by ID
pub async fn get_file(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<FileRow>, ApiError> { ... }

/// GET /api/v1/files/:id/symbols — List symbols for a file
pub async fn get_file_symbols(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<SymbolRow>>, ApiError> { ... }

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/files", get(list_files))
        .route("/files/{id}", get(get_file))
        .route("/files/{id}/symbols", get(get_file_symbols))
}
```

### 2. `src/routes/symbols.rs` — Symbol routes

```rust
#[derive(Deserialize)]
pub struct SymbolSearchQuery {
    pub q: Option<String>,
    pub kind: Option<String>,
}

/// GET /api/v1/symbols — List all symbols, optional ?kind= filter
pub async fn list_symbols(
    State(state): State<AppState>,
    Query(query): Query<SymbolSearchQuery>,
) -> Result<Json<Vec<SymbolRow>>, ApiError> { ... }

/// GET /api/v1/symbols/:id — Get a single symbol by ID
pub async fn get_symbol(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SymbolRow>, ApiError> { ... }

/// GET /api/v1/symbols/search?q=foo — Search symbols by name prefix
pub async fn search_symbols(
    State(state): State<AppState>,
    Query(query): Query<SymbolSearchQuery>,
) -> Result<Json<Vec<SymbolRow>>, ApiError> { ... }

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/symbols", get(list_symbols))
        .route("/symbols/search", get(search_symbols))
        .route("/symbols/{id}", get(get_symbol))
}
```

### 3. `src/routes/graph.rs` — Graph routes

```rust
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

/// GET /api/v1/graph — Full graph (nodes + edges)
pub async fn get_graph(State(state): State<AppState>) -> Result<Json<GraphResponse>, ApiError> { ... }

/// GET /api/v1/communities — List all communities
pub async fn list_communities(State(state): State<AppState>) -> Result<Json<Vec<CommunityResponse>>, ApiError> { ... }

/// GET /api/v1/processes — List all execution flows
pub async fn list_processes(State(state): State<AppState>) -> Result<Json<Vec<ProcessResponse>>, ApiError> { ... }

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/graph", get(get_graph))
        .route("/communities", get(list_communities))
        .route("/processes", get(list_processes))
}
```

### 4. `src/state.rs` — Shared application state

```rust
use codeilus_db::DbPool;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    pub db: Arc<DbPool>,
}

impl AppState {
    pub fn new(db: DbPool) -> Self {
        Self { db: Arc::new(db) }
    }
}
```

### 5. `src/error.rs` — API error type

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response, Json};
use serde_json::json;

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = json!({ "error": self.message });
        (self.status, Json(body)).into_response()
    }
}

impl From<codeilus_core::error::CodeilusError> for ApiError {
    fn from(err: codeilus_core::error::CodeilusError) -> Self {
        match &err {
            codeilus_core::error::CodeilusError::NotFound(_) => ApiError {
                status: StatusCode::NOT_FOUND,
                message: err.to_string(),
            },
            _ => ApiError {
                status: StatusCode::INTERNAL_SERVER_ERROR,
                message: err.to_string(),
            },
        }
    }
}
```

### 6. Update `src/routes/mod.rs` — Register all route modules

```rust
pub mod files;
pub mod graph;
pub mod health;
pub mod symbols;
pub mod ws;

use axum::Router;
use crate::state::AppState;

pub fn api_router() -> Router<AppState> {
    Router::new()
        .merge(health::router())
        .merge(files::router())
        .merge(symbols::router())
        .merge(graph::router())
}
```

### 7. Update `src/lib.rs` — Wire AppState and routes

Update the server builder to:
- Create `AppState` from `DbPool`
- Mount `api_router()` under `/api/v1`
- Keep existing rust-embed SPA fallback and WebSocket

## Tests

Create `crates/codeilus-api/tests/` with integration tests:

### Test cases:
1. `health_returns_ok` — GET /api/v1/health → 200 `{"status":"ok"}`
2. `list_files_empty` — GET /api/v1/files on empty DB → 200 `[]`
3. `list_files_with_data` — Insert 3 files via FileRepo, GET /api/v1/files → 3 items
4. `list_files_language_filter` — Insert Python + Rust files, GET /api/v1/files?language=python → only Python
5. `get_file_by_id` — Insert file, GET /api/v1/files/1 → correct file
6. `get_file_not_found` — GET /api/v1/files/999 → 404
7. `get_file_symbols` — Insert file + symbols, GET /api/v1/files/1/symbols → correct symbols
8. `list_symbols` — GET /api/v1/symbols → all symbols
9. `get_symbol_by_id` — GET /api/v1/symbols/1 → correct symbol
10. `search_symbols_prefix` — GET /api/v1/symbols/search?q=proc → matches "process"
11. `get_graph_empty` — GET /api/v1/graph on empty DB → `{"nodes":[],"edges":[]}`
12. `list_communities_empty` — GET /api/v1/communities → `[]`
13. `list_processes_empty` — GET /api/v1/processes → `[]`

Use `axum::test::TestServer` or build the router and call with `tower::ServiceExt::oneshot`.

## Acceptance Criteria

- [ ] `cargo test -p codeilus-api` — all tests pass
- [ ] `cargo clippy -p codeilus-api` — zero warnings
- [ ] All 9 routes respond with correct JSON
- [ ] 404 returned for missing resources
- [ ] Language filter works on GET /files
- [ ] Symbol search by prefix works
- [ ] Graph endpoint returns nodes + edges from DB
- [ ] Communities endpoint returns communities with member counts
- [ ] Processes endpoint returns flows with ordered steps

## Do NOT Touch
- `src/routes/health.rs` (done)
- `src/routes/ws.rs` (done)
- Any files outside `crates/codeilus-api/`
- `Cargo.toml` at workspace root
- Any other crate

---

## Report

> **Agent: fill this section when done.**

### Status: pending

### Files Created/Modified:
<!-- list all files you created/modified -->

### Tests:
<!-- paste `cargo test -p codeilus-api` output -->

### Clippy:
<!-- paste `cargo clippy -p codeilus-api` output -->

### Issues / Blockers:
<!-- any problems encountered -->

### Notes:
<!-- anything the next wave needs to know -->
