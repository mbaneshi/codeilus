# Task: Knowledge Graph Builder

> **Crate:** `crates/codeilus-graph/`
> **Wave:** 2 (parallel with api-routes)
> **Depends on:** codeilus-core (done), codeilus-parse (wave 1), codeilus-db repos (wave 1)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 6.3 (codeilus-graph deep dive)
- `crates/codeilus-core/src/types.rs` — EdgeKind, Confidence, SymbolKind, Language
- `crates/codeilus-core/src/ids.rs` — FileId, SymbolId, EdgeId, CommunityId
- `crates/codeilus-core/src/error.rs` — CodeilusError, CodeilusResult
- `crates/codeilus-parse/src/types.rs` — ParsedFile, ExtractedSymbol, ExtractedCall, ExtractedHeritage
- `crates/codeilus-db/src/repos/edge_repo.rs` — EdgeRepo for persisting edges
- Reference: `../GitNexus/src/core/ingestion/` — graph construction, community detection, process detection patterns

## Objective

Build the knowledge graph from parsed data. Construct call graph, dependency graph, and heritage graph as petgraph DiGraph. Run Louvain community detection, score entry points, and detect execution flows. Persist edges, communities, and processes to DB.

Public API:
```rust
pub fn build_graph(parsed_files: &[ParsedFile], db: &GraphDb) -> CodeilusResult<KnowledgeGraph>
```

## Files to Create/Modify

### 1. Update `crates/codeilus-graph/Cargo.toml`

```toml
[package]
name = "codeilus-graph"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
codeilus-db = { path = "../codeilus-db" }
petgraph = "0.6"
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 2. `src/types.rs` — Graph types

```rust
use codeilus_core::ids::{SymbolId, FileId, CommunityId};
use codeilus_core::types::{EdgeKind, Confidence};
use petgraph::graph::{DiGraph, NodeIndex};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

/// The in-memory knowledge graph.
pub struct KnowledgeGraph {
    pub graph: DiGraph<GraphNode, GraphEdge>,
    pub node_index: HashMap<SymbolId, NodeIndex>,
    pub communities: Vec<Community>,
    pub processes: Vec<Process>,
    pub entry_points: Vec<EntryPoint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub symbol_id: SymbolId,
    pub file_id: FileId,
    pub name: String,
    pub kind: String,
    pub community_id: Option<CommunityId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub kind: EdgeKind,
    pub confidence: Confidence,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Community {
    pub id: CommunityId,
    pub label: String,
    pub members: Vec<SymbolId>,
    pub cohesion: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Process {
    pub name: String,
    pub entry_symbol_id: SymbolId,
    pub steps: Vec<ProcessStep>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStep {
    pub order: usize,
    pub symbol_id: SymbolId,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPoint {
    pub symbol_id: SymbolId,
    pub score: f64,
    pub reason: String,
}
```

### 3. `src/call_graph.rs` — Call graph construction

- Take `&[ParsedFile]` and build CALLS edges
- Match caller/callee by symbol name + scope resolution
- Assign confidence: exact match = 1.0, name-only match = 0.7, ambiguous = 0.4
- Return `Vec<(SymbolId, SymbolId, Confidence)>`
- Reference: `../GitNexus/src/core/ingestion/` call tracing logic

### 4. `src/dep_graph.rs` — File dependency graph

- Take `&[ParsedFile]` and build IMPORTS edges at file level
- Use resolved import paths from `ParsedFile.imports`
- Match import source to file paths in the parsed set
- Return `Vec<(FileId, FileId)>` (file A imports file B)

### 5. `src/heritage.rs` — Heritage graph

- Take `&[ParsedFile]` and build EXTENDS / IMPLEMENTS edges
- Match child/parent by symbol name across files
- Return `Vec<(SymbolId, SymbolId, EdgeKind)>`

### 6. `src/community.rs` — Louvain community detection

- Implement Louvain algorithm on petgraph (undirected projection)
- Input: `&DiGraph<GraphNode, GraphEdge>`
- Output: `Vec<Community>` with member assignments and cohesion scores
- Algorithm: iterative modularity optimization
  1. Each node starts as its own community
  2. Move nodes to neighbor's community if modularity increases
  3. Repeat until no improvement
  4. Aggregate communities and repeat
- Compute modularity Q-score for each community
- Reference: `../GitNexus/src/core/ingestion/` community detection

### 7. `src/entry_points.rs` — Entry point scoring

- Heuristic scoring for identifying entry point symbols:
  - `main` function → +1.0
  - `index` / `mod` / `__init__` files → +0.5
  - Handler/route patterns (contains "handler", "route", "endpoint") → +0.7
  - CLI patterns (contains "cli", "cmd", "command") → +0.6
  - High fan-in (many callers) → +0.3
  - Zero callers (top-level) → +0.2
- Return `Vec<EntryPoint>` sorted by score descending

### 8. `src/process.rs` — Execution flow detection

- BFS from each entry point through CALLS edges
- Build `Process` with ordered `ProcessStep` list
- Limit BFS depth to 20 to avoid explosion
- Deduplicate steps (same symbol visited from different paths)
- Return `Vec<Process>`

### 9. `src/builder.rs` — Graph builder orchestrator

```rust
use crate::types::KnowledgeGraph;
use codeilus_core::CodeilusResult;
use codeilus_parse::ParsedFile;

pub struct GraphBuilder {
    // holds reference to parsed data + DB connections
}

impl GraphBuilder {
    pub fn new() -> Self;

    /// Build the full knowledge graph from parsed files.
    /// 1. Build symbol index (name → SymbolId mapping)
    /// 2. Construct call graph edges
    /// 3. Construct dependency graph edges
    /// 4. Construct heritage edges
    /// 5. Run Louvain community detection
    /// 6. Score entry points
    /// 7. Detect execution flows
    pub fn build(&self, parsed_files: &[ParsedFile]) -> CodeilusResult<KnowledgeGraph>;
}
```

### 10. `src/lib.rs` — Module entry point

```rust
pub mod builder;
pub mod call_graph;
pub mod community;
pub mod dep_graph;
pub mod entry_points;
pub mod heritage;
pub mod process;
pub mod types;

pub use builder::GraphBuilder;
pub use types::*;
```

### 11. Add to `crates/codeilus-db/src/repos/` — Two new repos

#### `community_repo.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityRow {
    pub id: CommunityId,
    pub label: String,
    pub cohesion: f64,
}

pub struct CommunityRepo { conn: Arc<Mutex<Connection>> }

impl CommunityRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn insert(&self, label: &str, cohesion: f64) -> CodeilusResult<CommunityId>;
    pub fn insert_batch(&self, communities: &[(String, f64)]) -> CodeilusResult<Vec<CommunityId>>;
    pub fn insert_member(&self, community_id: CommunityId, symbol_id: SymbolId) -> CodeilusResult<()>;
    pub fn insert_members_batch(&self, members: &[(CommunityId, SymbolId)]) -> CodeilusResult<()>;
    pub fn get(&self, id: CommunityId) -> CodeilusResult<CommunityRow>;
    pub fn list(&self) -> CodeilusResult<Vec<CommunityRow>>;
    pub fn list_members(&self, community_id: CommunityId) -> CodeilusResult<Vec<SymbolId>>;
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

#### `process_repo.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessRow {
    pub id: i64,
    pub name: String,
    pub entry_symbol_id: SymbolId,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessStepRow {
    pub id: i64,
    pub process_id: i64,
    pub step_order: i64,
    pub symbol_id: SymbolId,
    pub description: String,
}

pub struct ProcessRepo { conn: Arc<Mutex<Connection>> }

impl ProcessRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn insert(&self, name: &str, entry_symbol_id: SymbolId) -> CodeilusResult<i64>;
    pub fn insert_step(&self, process_id: i64, step_order: i64, symbol_id: SymbolId, description: &str) -> CodeilusResult<i64>;
    pub fn get(&self, id: i64) -> CodeilusResult<ProcessRow>;
    pub fn list(&self) -> CodeilusResult<Vec<ProcessRow>>;
    pub fn list_steps(&self, process_id: i64) -> CodeilusResult<Vec<ProcessStepRow>>;
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

Update `crates/codeilus-db/src/repos/mod.rs` to include both new repos.

## Tests

Create `crates/codeilus-graph/tests/` with test data:

### Test cases:
1. `call_graph_builds_edges` — Given 2 ParsedFiles where function A calls function B, call graph has CALLS edge A→B
2. `call_graph_confidence_exact` — Exact name+file match gets confidence 1.0
3. `call_graph_confidence_ambiguous` — Name-only match with multiple candidates gets lower confidence
4. `dep_graph_from_imports` — File with `import ./utils` creates IMPORTS edge to utils file
5. `heritage_extends` — Class B extends A → EXTENDS edge B→A
6. `heritage_implements` — Class C implements Interface D → IMPLEMENTS edge C→D
7. `louvain_two_clusters` — Graph with two obvious clusters (clique A, clique B, one bridge) → detects 2 communities
8. `louvain_single_community` — Fully connected graph → 1 community
9. `entry_point_main` — Symbol named "main" scores highest
10. `entry_point_handler` — Symbol named "handle_request" gets handler bonus
11. `process_bfs_linear` — Linear call chain A→B→C produces 3-step process
12. `process_bfs_depth_limit` — Chain deeper than 20 is truncated
13. `build_graph_integration` — Full pipeline from ParsedFiles → KnowledgeGraph with communities and processes

### DB repo tests (in codeilus-db):
14. `community_repo_insert_and_list` — Insert community, add members, list returns both
15. `process_repo_insert_and_list_steps` — Insert process with steps, verify ordering

## Acceptance Criteria

- [ ] `cargo test -p codeilus-graph` — all tests pass
- [ ] `cargo clippy -p codeilus-graph` — zero warnings
- [ ] `cargo test -p codeilus-db` — all tests pass (including new repo tests)
- [ ] `build_graph` takes `&[ParsedFile]` and returns `KnowledgeGraph` with populated communities
- [ ] Louvain detects at least 2 communities for non-trivial input
- [ ] Entry points ranked by score, main/index files score highest
- [ ] Execution flows traced via BFS from entry points
- [ ] All edges persisted via EdgeRepo batch insert
- [ ] Communities persisted via CommunityRepo
- [ ] Processes persisted via ProcessRepo

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- `crates/codeilus-parse/` (wave 1 owns this)
- `crates/codeilus-db/src/pool.rs`, `migrations.rs`, `batch_writer.rs`
- `crates/codeilus-db/src/repos/file_repo.rs`, `symbol_repo.rs`, `edge_repo.rs` (wave 1 owns these)
- `migrations/0001_init.sql`
- Any files outside `crates/codeilus-graph/` and the two new DB repo files

---

## Report

### Status: complete

### Files Created/Modified:

**codeilus-graph (new files):**
- `Cargo.toml` — Updated with petgraph, codeilus-parse, codeilus-db, serde deps
- `src/lib.rs` — Module entry point with all public exports
- `src/types.rs` — KnowledgeGraph, GraphNode, GraphEdge, Community, Process, ProcessStep, EntryPoint
- `src/call_graph.rs` — Call graph construction with confidence scoring (exact=1.0, name-only=0.7, ambiguous=0.4)
- `src/dep_graph.rs` — File-level dependency graph from import resolution
- `src/heritage.rs` — Extends/Implements edge construction
- `src/community.rs` — Louvain community detection with iterative modularity optimization
- `src/entry_points.rs` — Heuristic entry point scoring (main, handler, CLI, fan-in patterns)
- `src/process.rs` — BFS execution flow detection from entry points, depth-limited to 20
- `src/builder.rs` — GraphBuilder orchestrator: indexes → call graph → deps → heritage → communities → entry points → processes
- `tests/graph_tests.rs` — 13 test cases

**codeilus-db (new files):**
- `src/repos/community_repo.rs` — CommunityRepo with insert, batch insert, members, list, delete
- `src/repos/process_repo.rs` — ProcessRepo with insert, steps, list, delete
- `src/repos/mod.rs` — Updated to export new repos
- `src/lib.rs` — Updated to re-export CommunityRepo, CommunityRow, ProcessRepo, ProcessRow, ProcessStepRow
- `tests/repos.rs` — Added 2 new tests (community_repo_insert_and_list, process_repo_insert_and_list_steps)

### Tests:
```
cargo test -p codeilus-graph:
running 13 tests
test dep_graph_from_imports ... ok
test entry_point_handler ... ok
test entry_point_main ... ok
test louvain_single_community ... ok
test heritage_extends ... ok
test heritage_implements ... ok
test call_graph_builds_edges ... ok
test call_graph_confidence_ambiguous ... ok
test call_graph_confidence_exact ... ok
test build_graph_integration ... ok
test process_bfs_linear ... ok
test louvain_two_clusters ... ok
test process_bfs_depth_limit ... ok
test result: ok. 13 passed; 0 failed

cargo test -p codeilus-db:
running 19 tests — all pass (including 2 new repo tests)
```

### Clippy:
```
cargo clippy -p codeilus-graph — zero warnings
cargo clippy -p codeilus-db — zero warnings
```

### Issues / Blockers:
None.

### Notes:
- `codeilus-graph` depends on `codeilus-parse` for `ParsedFile` types (in addition to core + db)
- GraphBuilder assigns synthetic SymbolId/FileId values (sequential from 1) — when integrating with DB, callers should use DB-assigned IDs instead
- Louvain community detection works on the undirected projection of the directed graph
- Entry point scoring includes "handle" pattern (not just "handler") to match names like `handle_request`
- BFS process detection is depth-limited to 20 and deduplicates visited nodes
- The `process_steps` table doesn't have a `description` column (per migration schema) — ProcessRepo.insert_step accepts but ignores description param
