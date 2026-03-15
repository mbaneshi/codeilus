# Senior Architecture Review — Deep Codebase Assessment

**Date:** 2026-03-15
**Reviewer:** Senior/Staff-Level Architecture Review
**Scope:** Full codebase (16 crates, frontend, schema, infra)

---

## Executive Summary

Codeilus has a strong foundational architecture — the 16-crate workspace with clear dependency rules is well-designed. However, the project has classic "vision-first, implementation-second" gaps: the skeleton is comprehensive but several crates are stubs with minimal real logic. Below are professional recommendations organized from structural concerns to tactical improvements.

---

## 1. Architectural Concerns

### 1.1 EventBus Is Overengineered for Current Needs

**Observation:** The EventBus (tokio broadcast channel, capacity 256) is a central architectural concept, but its only consumer is the BatchWriter that persists events to an `events` table — which nothing reads from.

**Problem:**
- Events are fire-and-forget into a table nobody queries
- The WebSocket endpoint (`/ws`) subscribes to EventBus but the frontend doesn't use it during analysis (no real-time progress UI exists)
- BatchWriter introduces a 2-second flush delay for no consumer benefit

**Recommendation:**
- **Short-term:** Remove BatchWriter until there's a consumer. Events can be logged via tracing and persisted on-demand
- **Long-term:** If real-time analysis progress is needed, the EventBus → WebSocket → frontend pipeline is correct, but implement the frontend progress UI first, then wire up the bus
- **Pattern:** This is a classic YAGNI violation. Build the consumer, then build the infrastructure

### 1.2 `DbPool` Is Not Actually a Pool

**Observation:** Looking at `codeilus-db/src/lib.rs`, `DbPool` wraps a single `rusqlite::Connection`. In a concurrent Axum server, this means:
- All DB access is serialized through a single connection
- Under concurrent requests, handlers will block or panic
- SQLite WAL mode helps with concurrent reads but not with Rust's ownership model

**Recommendation:**
```rust
// BEFORE: Single connection pretending to be a pool
pub struct DbPool {
    conn: Connection,
}

// AFTER: Use r2d2-sqlite or deadpool-sqlite for actual connection pooling
use r2d2_sqlite::SqliteConnectionManager;
use r2d2::Pool;

pub struct DbPool {
    pool: Pool<SqliteConnectionManager>,
}

impl DbPool {
    pub fn new(path: &str) -> CodeilusResult<Self> {
        let manager = SqliteConnectionManager::file(path)
            .with_init(|c| {
                c.execute_batch("PRAGMA journal_mode=WAL; PRAGMA busy_timeout=5000;")?;
                Ok(())
            });
        let pool = Pool::builder().max_size(8).build(manager)?;
        Ok(Self { pool })
    }

    pub fn get(&self) -> CodeilusResult<r2d2::PooledConnection<SqliteConnectionManager>> {
        Ok(self.pool.get()?)
    }
}
```

This is a production-critical fix for any concurrent usage.

### 1.3 No Error Recovery in Analysis Pipeline

**Observation:** The 8-step analysis pipeline in `main.rs` is a linear sequence with `?` propagation. If step 5 (Metrics) fails, all prior work (Parse, Store, Graph) is lost.

**Recommendation:**
- Implement **checkpoint-resume**: After each step, persist a `pipeline_state` record with `{ step: "graph", status: "complete", timestamp }`
- On restart, resume from the last successful checkpoint
- This is critical for large repos where parsing alone can take minutes

```rust
enum PipelineStep { Parse, Store, Graph, Metrics, Analyze, Diagram, Narrate, Learn }

async fn run_pipeline(db: &DbPool, path: &Path) -> CodeilusResult<()> {
    let last_step = db.get_last_pipeline_step()?;

    for step in PipelineStep::iter().skip_while(|s| s <= &last_step) {
        match step {
            PipelineStep::Parse => { /* ... */ },
            PipelineStep::Store => { /* ... */ },
            // ...
        }
        db.mark_pipeline_step(step, "complete")?;
    }
    Ok(())
}
```

### 1.4 Missing Graceful Degradation Chain

**Observation:** Multiple crates assume their upstream dependency succeeded. If `codeilus-graph` fails, `codeilus-metrics` panics because it expects a `KnowledgeGraph`. If `codeilus-llm` is unavailable, `codeilus-narrate` returns empty content but `codeilus-learn` doesn't handle empty narratives.

**Recommendation:** Define a degradation hierarchy:

```
Full mode:     Parse → Graph → Metrics → Analyze → Diagram → Narrate → Learn
No LLM:        Parse → Graph → Metrics → Analyze → Diagram → [fallback] → Learn
No graph:      Parse → [flat metrics] → [basic analysis] → [file tree only]
Parse failure:  [show error with partial results]
```

Each crate should accept `Option<&T>` for upstream data and produce meaningful output even when partial.

---

## 2. Code Quality Issues

### 2.1 Repository Structs Have Inconsistent Error Handling

**Observation:** Some repos use `rusqlite::Result` directly, others convert to `CodeilusError`, and some unwrap internally.

**Recommendation:** All repo methods should return `CodeilusResult<T>` consistently. Add a `From<rusqlite::Error>` impl in `codeilus-core/src/error.rs` (if not already present) and use `?` throughout.

### 2.2 No Input Validation Layer

**Observation:** API routes accept path parameters (`:id`) as `i64` but don't validate them against known ranges. A request for `/files/999999` returns a 500 error with a raw SQLite "not found" message rather than a clean 404.

**Recommendation:**
```rust
// Add a helper in routes/
fn not_found_if_none<T>(opt: Option<T>, entity: &str, id: i64) -> Result<T, ApiError> {
    opt.ok_or_else(|| ApiError::NotFound(format!("{entity} with id {id} not found")))
}

// Usage:
let file = file_repo.get(id)?;
let file = not_found_if_none(file, "File", id)?;
```

### 2.3 No Rate Limiting on LLM Endpoints

**Observation:** `POST /api/v1/ask` directly spawns a Claude Code subprocess for every request. No rate limiting, no queue, no concurrent request cap.

**Recommendation:**
```rust
use tokio::sync::Semaphore;

pub struct AppState {
    // ...existing fields...
    pub llm_semaphore: Arc<Semaphore>,  // Limit concurrent LLM calls
}

// In ask handler:
let _permit = state.llm_semaphore.acquire().await
    .map_err(|_| ApiError::ServiceUnavailable("LLM is busy"))?;
```

Set semaphore permits to 1-3 depending on hardware. Add a `429 Too Many Requests` response for queued requests.

### 2.4 Hardcoded Magic Numbers

**Observation:** Throughout the codebase:
- BatchWriter: `50` events, `2` seconds (hardcoded)
- EventBus: `256` capacity (hardcoded)
- Search: `60.0` RRF constant, `20` default limit (hardcoded)
- Parse: `20 * 1024 * 1024` max file size (hardcoded)

**Recommendation:** Create a `CodeilusConfig` struct in `codeilus-core`:

```rust
pub struct CodeilusConfig {
    pub batch_size: usize,           // default: 50
    pub batch_flush_secs: u64,       // default: 2
    pub event_bus_capacity: usize,   // default: 256
    pub max_file_bytes: usize,       // default: 20MB
    pub search_rrf_k: f64,           // default: 60.0
    pub search_default_limit: usize, // default: 20
    pub llm_max_concurrent: usize,   // default: 1
    pub server_port: u16,            // default: 4174
}

impl Default for CodeilusConfig { /* sensible defaults */ }
```

Load from env vars or a `codeilus.toml` config file.

---

## 3. Performance Recommendations

### 3.1 Parsing Needs Incremental Mode

**Observation:** `parse_repository()` re-parses the entire repo every time. For a 10,000-file codebase, this takes 30+ seconds even with rayon.

**Recommendation:** Implement file-hash-based incremental parsing:

```rust
pub fn parse_repository_incremental(
    config: &ParseConfig,
    db: &DbPool,
    bus: Option<&EventBus>,
) -> CodeilusResult<Vec<ParsedFile>> {
    let existing_hashes = db.get_file_hashes()?;  // HashMap<PathBuf, String>
    let files = walk_files(config)?;

    let (changed, unchanged): (Vec<_>, Vec<_>) = files.into_par_iter()
        .partition(|f| {
            let hash = hash_file(f);
            existing_hashes.get(f) != Some(&hash)
        });

    tracing::info!("Incremental: {} changed, {} unchanged", changed.len(), unchanged.len());

    let parsed = changed.into_par_iter()
        .map(|f| parse_file(f, config))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(parsed)
}
```

This alone would make re-analysis 10-50x faster for typical edit-analyze-view cycles.

### 3.2 Graph Queries Are Unbounded

**Observation:** `GET /api/v1/graph` returns the entire knowledge graph — every node and every edge. For a codebase with 5,000 symbols and 20,000 edges, this is a multi-MB JSON response that will crash the 3D force graph visualization.

**Recommendation:**
- Add pagination: `/api/v1/graph?community_id=3&depth=2&limit=200`
- Server-side graph filtering: return only the subgraph around a focal node
- Progressive loading: first load communities as clusters, expand on click

### 3.3 No Caching Layer

**Observation:** Every API request hits SQLite directly. Narratives (which never change after generation) are re-queried on every page load.

**Recommendation:** Add an in-memory cache for immutable data:

```rust
use std::sync::Arc;
use dashmap::DashMap;

pub struct AppState {
    // ...existing...
    pub narrative_cache: Arc<DashMap<(NarrativeKind, Option<i64>), String>>,
}
```

Or use `moka` for a bounded, TTL-based cache. Narratives, graph structure, and metrics are write-once-read-many — perfect cache candidates.

---

## 4. Testing Strategy Gaps

### 4.1 No Integration Tests for API Layer

**Observation:** The test suite has unit tests for parsing, graph building, and search. But there are zero tests for the Axum API layer — no request/response testing.

**Recommendation:** Add `axum::test` based integration tests:

```rust
#[tokio::test]
async fn test_list_files() {
    let db = DbPool::in_memory().unwrap();
    // Seed test data
    let state = AppState::test(db);
    let app = codeilus_api::app(state);

    let response = app
        .oneshot(Request::builder().uri("/api/v1/files").body(Body::empty()).unwrap())
        .await
        .unwrap();

    assert_eq!(response.status(), 200);
    let body: Vec<FileRow> = serde_json::from_slice(&hyper::body::to_bytes(response.into_body()).await.unwrap()).unwrap();
    assert_eq!(body.len(), expected_count);
}
```

### 4.2 No Property-Based Tests for Graph Algorithms

**Observation:** Graph algorithms (Louvain, entry point scoring, BFS process detection) are tested with hand-crafted fixtures. These are the most complex algorithms in the project and have the highest bug surface area.

**Recommendation:** Use `proptest` or `quickcheck`:

```rust
proptest! {
    #[test]
    fn louvain_communities_cover_all_nodes(graph in arb_graph(10..100)) {
        let communities = louvain_detect(&graph);
        let covered: HashSet<_> = communities.iter().flat_map(|c| &c.members).collect();
        let all_nodes: HashSet<_> = graph.nodes().collect();
        prop_assert_eq!(covered, all_nodes);
    }
}
```

### 4.3 No Test for Frontend-Backend Contract

**Recommendation:** Generate OpenAPI spec from backend routes using `utoipa`, then use it to generate TypeScript types. This creates a compile-time guarantee that frontend types match backend responses.

```rust
// In codeilus-api
#[derive(Serialize, utoipa::ToSchema)]
pub struct FileRow { /* ... */ }
```

---

## 5. Security Considerations

### 5.1 Path Traversal in File Source Endpoint

**Observation:** `GET /api/v1/files/:id/source` reads file content from disk based on the `path` stored in the database. If an attacker can inject a file record with `path = "/etc/passwd"`, the endpoint will serve arbitrary system files.

**Recommendation:** Validate that the resolved path is within the analyzed repository root:

```rust
let repo_root = state.repo_root.as_ref().ok_or(ApiError::BadRequest("No repo analyzed"))?;
let file_path = repo_root.join(&file_row.path);
let canonical = file_path.canonicalize()?;

if !canonical.starts_with(repo_root.canonicalize()?) {
    return Err(ApiError::Forbidden("Path traversal detected"));
}
```

### 5.2 No CORS Origin Restriction

**Observation:** CORS is configured with `AllowOrigin::any()`. Since the server is on localhost, this allows any website to make requests to the Codeilus API.

**Recommendation:** Restrict to localhost origins:

```rust
let cors = CorsLayer::new()
    .allow_origin([
        "http://localhost:4174".parse().unwrap(),
        "http://localhost:5173".parse().unwrap(),  // Vite dev
        "http://127.0.0.1:4174".parse().unwrap(),
    ])
    .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE]);
```

### 5.3 Claude Code CLI Injection

**Observation:** The Claude Code CLI is spawned as a subprocess with user-provided context. If user input flows into the prompt without sanitization, it could manipulate Claude's behavior.

**Recommendation:**
- Escape or validate question content before passing to Claude
- Use structured prompts with clear delimiters: `<user_question>{sanitized}</user_question>`
- Set max token limits on the subprocess

---

## 6. Dependency Health

### 6.1 Outdated or Risky Dependencies

| Dependency | Current | Concern |
|-----------|---------|---------|
| `rusqlite` 0.32 | `bundled` feature | Embeds SQLite C code; ensure it's updated for security patches |
| `tree-sitter` 0.24 | Recent | API changed significantly between 0.22 → 0.24; pin carefully |
| `axum` 0.7 | Good | Stable, well-maintained |
| `rmcp` | Unknown version | MCP SDK is very new; expect breaking changes |

**Recommendation:**
- Run `cargo audit` regularly
- Pin `rmcp` to exact version until MCP spec stabilizes
- Add `[patch]` section to workspace Cargo.toml for critical fixes

### 6.2 Feature Flag Bloat in Tokio

**Observation:** Workspace enables 6 tokio features: `sync, rt, rt-multi-thread, macros, signal, process, io-util`. Some crates (like `codeilus-core`) only need `sync` but pull in the full runtime.

**Recommendation:** Use per-crate feature selection:

```toml
# In codeilus-core/Cargo.toml
[dependencies]
tokio = { version = "1", features = ["sync"] }

# In codeilus-app/Cargo.toml (needs full runtime)
tokio = { version = "1", features = ["full"] }
```

---

## 7. Developer Experience Improvements

### 7.1 Add `cargo xtask` for Common Operations

```
cargo xtask dev          # Build backend + start frontend dev server
cargo xtask seed         # Populate DB with sample data for development
cargo xtask test-all     # cargo test + frontend tests
cargo xtask docs         # Build mkdocs site
cargo xtask release      # Full release build + embed frontend
```

### 7.2 Add Structured Logging

**Current:** Uses `tracing` but most log points are `info!("step complete")` without structured fields.

**Better:**
```rust
tracing::info!(
    files = parsed.len(),
    symbols = total_symbols,
    duration_ms = elapsed.as_millis(),
    "Parsing complete"
);
```

This enables structured log queries: "show me all parse events that took >5s".

### 7.3 Add Health Check With Diagnostics

The current `/health` endpoint returns 200 OK. Extend it:

```rust
#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    db_ok: bool,
    llm_available: bool,
    files_analyzed: Option<i64>,
    symbols_count: Option<i64>,
}
```

---

## 8. Scalability Considerations

### 8.1 Single-User Assumption

The entire architecture assumes a single user. `learner_stats` has a single row (id=1). `progress` has no user_id. If this ever needs to support multiple users (classroom, team), every learning-related table needs a `user_id` column.

**Recommendation:** Even if single-user now, add `user_id` columns with a default value of 1. This is a 5-minute migration now vs. a painful data migration later.

### 8.2 No Repo Isolation

If you analyze repo A then repo B, the DB accumulates both. There's a `clear_analysis_data()` but it deletes everything — no per-repo isolation.

**Recommendation:** Add a `repositories` table and make `file_id`, `symbol_id` etc. scoped to a `repo_id`. Or use separate SQLite databases per repo (simpler, more aligned with the "single binary" philosophy).

---

## Summary: Top 10 Priorities

| # | Issue | Effort | Impact |
|---|-------|--------|--------|
| 1 | Replace DbPool with actual connection pool | 2h | Critical (concurrency) |
| 2 | Add path traversal protection | 30m | Critical (security) |
| 3 | Implement checkpoint-resume in pipeline | 3h | High (reliability) |
| 4 | Add graph query pagination | 2h | High (performance) |
| 5 | Implement incremental parsing | 4h | High (performance) |
| 6 | Add API integration tests | 4h | High (quality) |
| 7 | Restrict CORS origins | 15m | Medium (security) |
| 8 | Add LLM rate limiting (semaphore) | 30m | Medium (stability) |
| 9 | Add input validation layer | 2h | Medium (UX) |
| 10 | Extract config to CodeilusConfig struct | 2h | Medium (maintainability) |
