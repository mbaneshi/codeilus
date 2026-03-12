# Task: MCP Server + Pipeline Wiring

> **Crates:** `crates/codeilus-mcp/` + `crates/codeilus-app/`
> **Wave:** 6 (solo — after all other waves)
> **Depends on:** ALL other crates
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 9, Sprint 8 (MCP + Polish), UC-5 (AI Agent Integration)
- `crates/codeilus-app/src/main.rs` — existing CLI with clap subcommands
- `crates/codeilus-core/src/types.rs` — all shared types
- `crates/codeilus-db/src/repos/` — all repo modules
- `crates/codeilus-graph/src/types.rs` — KnowledgeGraph
- All crate `src/lib.rs` files — understand the public APIs of every crate
- rmcp crate documentation: https://docs.rs/rmcp — Rust MCP SDK for stdio transport

## Objective

Two deliverables:

1. **MCP Server** (`codeilus-mcp`): Implement 8 MCP tools that AI agents (Claude Code, Cursor) can call to query the codebase knowledge graph via stdio transport.

2. **Pipeline Wiring** (`codeilus-app`): Wire the full analysis pipeline in `main.rs`: parse → store → graph → metrics → analyze → diagram → narrate → learn → serve.

## Files to Create/Modify

### Part 1: MCP Server (crates/codeilus-mcp/)

#### 1. Update `crates/codeilus-mcp/Cargo.toml`

```toml
[package]
name = "codeilus-mcp"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
codeilus-db = { path = "../codeilus-db" }
codeilus-graph = { path = "../codeilus-graph" }
codeilus-search = { path = "../codeilus-search" }
rmcp = { version = "0.1", features = ["server", "transport-io"] }
tokio = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

#### 2. `src/types.rs` — MCP tool input/output types

```rust
use serde::{Serialize, Deserialize};

// Input types for each tool
#[derive(Debug, Deserialize)]
pub struct QuerySymbolsInput {
    pub query: String,
    pub kind: Option<String>,      // filter by SymbolKind
    pub limit: Option<usize>,       // default 20
}

#[derive(Debug, Deserialize)]
pub struct QueryGraphInput {
    pub symbol_id: Option<i64>,     // if provided, return neighbors only
    pub depth: Option<usize>,       // BFS depth (default 1)
    pub edge_kind: Option<String>,  // filter by edge kind
}

#[derive(Debug, Deserialize)]
pub struct GetContextInput {
    pub focus: String,              // "overview", "symbol", "community", "files"
    pub target_id: Option<i64>,     // symbol_id or community_id
    pub file_paths: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
pub struct GetImpactInput {
    pub symbol_id: i64,
    pub depth: Option<usize>,       // blast radius depth (default 3)
}

#[derive(Debug, Deserialize)]
pub struct GetDiagramInput {
    pub kind: String,               // "architecture" or "flowchart"
    pub symbol_id: Option<i64>,     // required for flowchart
}

#[derive(Debug, Deserialize)]
pub struct GetMetricsInput {
    pub file_id: Option<i64>,       // specific file, or all
    pub top_hotspots: Option<usize>,// return top N hotspot files
}

#[derive(Debug, Deserialize)]
pub struct GetLearningStatusInput {
    // no required fields — returns overall progress
}

#[derive(Debug, Deserialize)]
pub struct ExplainSymbolInput {
    pub symbol_id: i64,
}
```

#### 3. `src/tools.rs` — MCP tool implementations

Implement 8 tools using rmcp's tool registration API:

```rust
use codeilus_db::DbPool;
use std::sync::Arc;

pub struct CodeilusTools {
    db: Arc<DbPool>,
}

impl CodeilusTools {
    pub fn new(db: Arc<DbPool>) -> Self;

    /// 1. query_symbols: Search symbols by name, filter by kind
    ///    Returns: symbol name, kind, file path, line range
    pub async fn query_symbols(&self, input: QuerySymbolsInput) -> serde_json::Value;

    /// 2. query_graph: Get graph neighborhood for a symbol
    ///    Returns: node + edges (callers, callees, dependencies)
    pub async fn query_graph(&self, input: QueryGraphInput) -> serde_json::Value;

    /// 3. get_context: Build context string for LLM consumption
    ///    Returns: structured context text (<8K tokens)
    pub async fn get_context(&self, input: GetContextInput) -> serde_json::Value;

    /// 4. get_impact: Blast radius analysis for a symbol
    ///    Returns: affected symbols with depth scores
    pub async fn get_impact(&self, input: GetImpactInput) -> serde_json::Value;

    /// 5. get_diagram: Generate Mermaid diagram
    ///    Returns: Mermaid syntax string
    pub async fn get_diagram(&self, input: GetDiagramInput) -> serde_json::Value;

    /// 6. get_metrics: File metrics and hotspots
    ///    Returns: metrics data (SLOC, complexity, churn, heatmap)
    pub async fn get_metrics(&self, input: GetMetricsInput) -> serde_json::Value;

    /// 7. get_learning_status: Current learning progress
    ///    Returns: XP, level, badges, chapter progress, streak
    pub async fn get_learning_status(&self, input: GetLearningStatusInput) -> serde_json::Value;

    /// 8. explain_symbol: Get or generate explanation for a symbol
    ///    Returns: narrative explanation text
    pub async fn explain_symbol(&self, input: ExplainSymbolInput) -> serde_json::Value;
}
```

Tool descriptions (for MCP tool registration):
1. `query_symbols` — "Search for symbols (functions, classes, etc.) by name. Returns matching symbols with file locations."
2. `query_graph` — "Query the knowledge graph for a symbol's relationships: callers, callees, dependencies, inheritance."
3. `get_context` — "Build structured context about the codebase for understanding. Focus on overview, a specific symbol, community, or file set."
4. `get_impact` — "Analyze the blast radius of changing a symbol. Returns all affected downstream symbols with risk scores."
5. `get_diagram` — "Generate a Mermaid diagram. 'architecture' for system overview, 'flowchart' for function control flow."
6. `get_metrics` — "Get code metrics: SLOC, complexity, churn, contributors, hotspot scores."
7. `get_learning_status` — "Get current learning progress: XP, level, badges, chapter completion, streak."
8. `explain_symbol` — "Get a human-readable explanation of what a symbol does, including its role and connections."

#### 4. `src/server.rs` — MCP server setup

```rust
use crate::tools::CodeilusTools;
use codeilus_db::DbPool;
use std::sync::Arc;

/// Start the MCP server on stdio.
pub async fn start_mcp_server(db: DbPool) -> codeilus_core::CodeilusResult<()> {
    let tools = CodeilusTools::new(Arc::new(db));

    // Register all 8 tools with rmcp
    // Use rmcp's ServerBuilder or equivalent API
    // Start stdio transport
    // Handle incoming tool calls by dispatching to CodeilusTools methods

    // The server runs until stdin is closed
    Ok(())
}
```

Use rmcp's `#[tool]` macro or manual registration. Follow rmcp crate patterns for:
- Server initialization with stdio transport
- Tool registration with name, description, input schema
- Request handling loop
- Graceful shutdown on stdin close

#### 5. `src/lib.rs` — Module entry point

```rust
pub mod server;
pub mod tools;
pub mod types;

pub use server::start_mcp_server;
```

### Part 2: Pipeline Wiring (crates/codeilus-app/)

#### 6. Update `crates/codeilus-app/src/main.rs` — Wire the full pipeline

The `analyze` subcommand should run the full pipeline:

```rust
async fn run_analyze(path: &Path, db: &DbPool) -> CodeilusResult<()> {
    // 1. PARSE
    tracing::info!("Parsing repository...");
    let parsed_files = codeilus_parse::parse_repository(path)?;
    tracing::info!(files = parsed_files.len(), "Parsing complete");

    // 2. STORE parsed data
    tracing::info!("Storing parsed data...");
    let conn = db.conn_arc();
    let file_repo = FileRepo::new(conn.clone());
    let symbol_repo = SymbolRepo::new(conn.clone());
    // Insert files and symbols into DB
    // Map ParsedFile data to DB rows

    // 3. GRAPH
    tracing::info!("Building knowledge graph...");
    let graph = codeilus_graph::GraphBuilder::new().build(&parsed_files)?;
    // Store edges, communities, processes via repos

    // 4. METRICS (parallel with analyze + diagram)
    tracing::info!("Computing metrics...");
    let metrics = codeilus_metrics::compute_metrics(&parsed_files, &graph, path)?;
    // Store metrics via FileMetricsRepo

    // 5. ANALYZE
    tracing::info!("Detecting patterns...");
    let patterns = codeilus_analyze::analyze(&parsed_files, &graph)?;
    // Store patterns via PatternRepo

    // 6. DIAGRAM
    tracing::info!("Generating diagrams...");
    let arch_diagram = codeilus_diagram::generate_architecture(&graph)?;
    // Store diagram in narratives or separate table

    // 7. NARRATE
    tracing::info!("Generating narratives...");
    let narratives = codeilus_narrate::generate_all_narratives(&graph, &parsed_files, path).await?;
    // Store narratives via NarrativeRepo

    // 8. LEARN
    tracing::info!("Building curriculum...");
    let curriculum = codeilus_learn::generate_curriculum(&graph)?;
    // Store chapters via ChapterRepo

    // 9. SERVE (if --serve flag)
    tracing::info!("Analysis complete. Ready to serve.");
    Ok(())
}
```

#### 7. Update `crates/codeilus-app/src/main.rs` — Wire MCP subcommand

```rust
async fn run_mcp(db: DbPool) -> CodeilusResult<()> {
    codeilus_mcp::start_mcp_server(db).await
}
```

#### 8. Update `crates/codeilus-app/src/main.rs` — Wire harvest + export subcommands

```rust
async fn run_harvest(config: HarvestConfig, db: &DbPool) -> CodeilusResult<()> {
    let repos = codeilus_harvest::harvest_trending(config).await?;
    // For each cloned repo: run_analyze
    // Store harvest status
    Ok(())
}

fn run_export(repo_name: &str, db: &DbPool, output_dir: &Path) -> CodeilusResult<()> {
    let path = codeilus_export::export_repo(repo_name, db, output_dir)?;
    tracing::info!(path = %path.display(), "Exported");
    Ok(())
}
```

## Tests

### MCP tests:
1. `query_symbols_returns_results` — Insert symbols in DB, call query_symbols → returns matches
2. `query_symbols_empty` — Empty DB → returns empty array
3. `query_symbols_kind_filter` — Filter by "function" → only functions returned
4. `query_graph_neighbors` — Insert symbol + edges, query_graph → returns neighbors
5. `get_impact_blast_radius` — Symbol with 3 downstream callers at depth 2 → all 3 returned
6. `get_diagram_architecture` — Get diagram kind="architecture" → returns valid Mermaid string
7. `get_metrics_hotspots` — Get top 5 hotspots → returns 5 entries sorted by score
8. `get_learning_status_initial` — Fresh DB → returns 0 XP, 0 badges
9. `explain_symbol_from_db` — Symbol with pre-generated explanation → returns it
10. `tool_descriptions_complete` — All 8 tools have non-empty descriptions

### Pipeline tests:
11. `analyze_pipeline_smoke` — Run full pipeline on test fixtures → DB populated with files, symbols, edges, communities
12. `analyze_pipeline_idempotent` — Run pipeline twice → no errors, DB state is clean (not duplicated)
13. `mcp_subcommand_exists` — CLI parses `codeilus mcp` without error

## Acceptance Criteria

### MCP Server:
- [ ] `cargo test -p codeilus-mcp` — all tests pass
- [ ] `cargo clippy -p codeilus-mcp` — zero warnings
- [ ] All 8 tools registered with rmcp
- [ ] Each tool has name, description, and typed input schema
- [ ] Tools return JSON responses
- [ ] Server runs on stdio transport
- [ ] Server handles graceful shutdown

### Pipeline Wiring:
- [ ] `cargo test -p codeilus-app` — all tests pass
- [ ] `cargo clippy -p codeilus-app` — zero warnings
- [ ] `codeilus analyze ./path` runs full pipeline: parse → store → graph → metrics → analyze → diagram → narrate → learn
- [ ] `codeilus mcp` starts MCP server on stdio
- [ ] `codeilus serve` starts HTTP server with all data available
- [ ] `codeilus harvest --trending` scrapes and clones repos
- [ ] `codeilus export --repo <name>` generates static HTML
- [ ] Pipeline logs progress via tracing at each stage
- [ ] Pipeline handles errors gracefully (one stage failing doesn't crash the rest)

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- Any crate other than `codeilus-mcp/` and `codeilus-app/` (read their public APIs, don't modify)
- `migrations/0001_init.sql`
- `frontend/` directory

---

## Report

> **Agent: fill this section when done.**

### Status: pending

### Files Created/Modified:
<!-- list all files you created/modified -->

### Tests:
<!-- paste `cargo test -p codeilus-mcp` output -->
<!-- paste `cargo test -p codeilus-app` output -->

### Clippy:
<!-- paste `cargo clippy -p codeilus-mcp` output -->
<!-- paste `cargo clippy -p codeilus-app` output -->

### Issues / Blockers:
<!-- any problems encountered -->

### Notes:
<!-- final integration notes -->
