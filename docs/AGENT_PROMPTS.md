# Codeilus — Parallel Agent Prompts

> Copy-paste these into Cursor agents. Designed so agents work on separate crates and don't conflict.

## How to Use

1. Open the project in Cursor: `cursor /Users/bm/codeilus/codeilus`
2. Each section below is one agent prompt — copy the entire block into a new Cursor agent session
3. Launch agents in **waves** — agents within the same wave are safe to run in parallel
4. Wait for a wave to complete before starting the next (later waves depend on earlier ones)
5. After each wave: `cargo test && cargo clippy` to verify

## Current State

Sprint 0 is complete. The following exists and works:

```
codeilus-core:  EventBus, 18 events, 12 errors, 5 ID types, Language/SymbolKind/EdgeKind/Confidence/NarrativeKind
codeilus-db:    DbPool (WAL), Migrator, BatchWriter, 20-table schema (files, symbols, edges, communities, narratives, learning, harvest, events)
codeilus-api:   Axum + CORS + WS + rust-embed SPA fallback, GET /health, GET /ws
codeilus-app:   clap CLI with analyze/serve/harvest/export/deploy/mcp subcommands
+ 12 stub crates (empty lib.rs)
```

All 16 crates compile. `cargo test` passes (10 tests). Zero clippy warnings.

---

## Wave 1: Sprint 1 — Parsing + DB Repos + Frontend Skeleton (3 agents)

These 3 agents work on completely separate files. Run them all in parallel.

### Agent 1A: codeilus-parse (Tree-sitter Parsing Engine)

```
You are implementing the codeilus-parse crate — the Tree-sitter-based parsing engine.

## Context
Read CLAUDE.md and NORTH_STAR.md for full project context. Read the reference repo at ../GitNexus/src/core/ingestion/ for parsing patterns.

## Your Crate
`crates/codeilus-parse/` — currently has an empty lib.rs stub.

## What to Build

### Cargo.toml Dependencies
Add to crates/codeilus-parse/Cargo.toml:
- codeilus-core = { path = "../codeilus-core" }
- tree-sitter = "0.24"
- tree-sitter-python = "0.23"
- tree-sitter-typescript = "0.23"
- tree-sitter-javascript = "0.23"
- tree-sitter-rust = "0.23"
- tree-sitter-go = "0.23"
- tree-sitter-java = "0.23"
- ignore = "0.4" (for .gitignore-respecting file walker)
- rayon = "1"
- tracing = { workspace = true }
- serde = { workspace = true }
- serde_json = { workspace = true }

### Source Files to Create

1. `src/lib.rs` — public API: `pub fn parse_repository(path: &Path) -> CodeilusResult<Vec<ParsedFile>>`
2. `src/walker.rs` — Walk filesystem respecting .gitignore, filter by known extensions, 20MB byte budget
3. `src/language.rs` — Map Language enum to tree-sitter grammar, create Parser per language
4. `src/extractor.rs` — Given a tree-sitter Tree + source bytes, extract symbols, imports, calls, heritage
5. `src/queries/mod.rs` — Tree-sitter query strings per language (Python, TS, JS, Rust, Go, Java)
6. `src/queries/python.rs` — Python-specific queries for functions, classes, imports, calls
7. `src/queries/typescript.rs` — TS/JS queries (shared base)
8. `src/queries/rust.rs` — Rust queries (fn, struct, impl, use, mod)
9. `src/queries/go.rs` — Go queries (func, type, import)
10. `src/queries/java.rs` — Java queries (class, method, import)
11. `src/resolver.rs` — Import path resolution (language-specific: Python relative, TS tsconfig, Rust mod)
12. `src/types.rs` — ParsedFile, Symbol, Import, Call, Heritage structs

### Key Types (in src/types.rs)
```rust
use codeilus_core::types::{Language, SymbolKind};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub path: String,
    pub language: Language,
    pub sloc: usize,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
    pub calls: Vec<Call>,
    pub heritage: Vec<Heritage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub start_line: usize,
    pub end_line: usize,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Import {
    pub source: String,        // "os.path" or "./utils"
    pub names: Vec<String>,    // ["join", "dirname"] or ["*"]
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Call {
    pub caller: String,        // enclosing function name
    pub callee: String,        // called function name
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Heritage {
    pub child: String,
    pub parent: String,
    pub kind: HeritageKind,    // Extends or Implements
}
```

### Tests
- Create `tests/fixtures/` with small sample files (Python, TS, Rust)
- Test that parse_repository finds files, extracts symbols correctly
- Test import resolution for each language
- Test SLOC counting

### Rules
- Only modify files inside crates/codeilus-parse/
- Use codeilus_core types (Language, SymbolKind, etc.) — don't redefine
- Rayon for parallelism, not tokio (CPU-bound work)
- Run `cargo test -p codeilus-parse && cargo clippy -p codeilus-parse` before finishing
```

### Agent 1B: codeilus-db Repos (File + Symbol Repositories)

```
You are implementing the repository layer in codeilus-db for files and symbols.

## Context
Read CLAUDE.md and NORTH_STAR.md. Read crates/codeilus-db/src/ to understand existing code (pool.rs, migrations.rs, batch_writer.rs). Read migrations/0001_init.sql for the schema.

## What to Build

### Source Files to Create/Modify

1. `crates/codeilus-db/src/repos/mod.rs` — Replace stub with: pub mod file_repo; pub mod symbol_repo; pub mod edge_repo; pub mod narrative_repo;
2. `crates/codeilus-db/src/repos/file_repo.rs` — CRUD for files table
3. `crates/codeilus-db/src/repos/symbol_repo.rs` — CRUD for symbols table
4. `crates/codeilus-db/src/repos/edge_repo.rs` — CRUD for edges table (stub, filled in Sprint 2)
5. `crates/codeilus-db/src/repos/narrative_repo.rs` — CRUD for narratives table (stub, filled in Sprint 5)
6. Update `crates/codeilus-db/src/lib.rs` to re-export new repos

### FileRepo API
```rust
pub struct FileRepo { conn: Arc<Mutex<Connection>> }

impl FileRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn insert(&self, path: &str, language: Option<&str>, sloc: i64) -> CodeilusResult<FileId>;
    pub fn insert_batch(&self, files: &[(String, Option<String>, i64)]) -> CodeilusResult<Vec<FileId>>;
    pub fn get(&self, id: FileId) -> CodeilusResult<FileRow>;
    pub fn get_by_path(&self, path: &str) -> CodeilusResult<Option<FileRow>>;
    pub fn list(&self) -> CodeilusResult<Vec<FileRow>>;
    pub fn count(&self) -> CodeilusResult<usize>;
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

### SymbolRepo API
```rust
pub struct SymbolRepo { conn: Arc<Mutex<Connection>> }

impl SymbolRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn insert(&self, file_id: FileId, name: &str, kind: &str, start_line: i64, end_line: i64, signature: Option<&str>) -> CodeilusResult<SymbolId>;
    pub fn insert_batch(&self, symbols: &[SymbolRow]) -> CodeilusResult<Vec<SymbolId>>;
    pub fn get(&self, id: SymbolId) -> CodeilusResult<SymbolRow>;
    pub fn list_by_file(&self, file_id: FileId) -> CodeilusResult<Vec<SymbolRow>>;
    pub fn list_by_name(&self, name: &str) -> CodeilusResult<Vec<SymbolRow>>;
    pub fn count(&self) -> CodeilusResult<usize>;
}
```

### Row Types
```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRow {
    pub id: FileId,
    pub path: String,
    pub language: Option<String>,
    pub sloc: i64,
    pub last_modified: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRow {
    pub id: SymbolId,
    pub file_id: FileId,
    pub name: String,
    pub kind: String,
    pub start_line: i64,
    pub end_line: i64,
    pub signature: Option<String>,
}
```

### Tests
- Use DbPool::in_memory() + Migrator for all tests
- Test insert + get roundtrip for files
- Test batch insert for files and symbols
- Test list_by_file returns correct symbols
- Test get_by_path works
- Test count

### Rules
- Only modify files inside crates/codeilus-db/
- All methods use unchecked_transaction() for batch operations
- Use rusqlite::params! macro
- Run `cargo test -p codeilus-db && cargo clippy -p codeilus-db` before finishing
```

### Agent 1C: Frontend Skeleton (SvelteKit 5 + TailwindCSS 4)

```
You are creating the frontend skeleton for Codeilus using SvelteKit 5 with adapter-static and TailwindCSS 4.

## Context
Read CLAUDE.md and NORTH_STAR.md. The frontend will be embedded into the Rust binary via rust-embed. The API runs at the same origin (no CORS issues in prod). The rust-embed directive points to `frontend/build/`.

## What to Build

### Setup
```bash
cd /Users/bm/codeilus/codeilus
npx sv create frontend  # choose: SvelteKit skeleton, TypeScript, adapter-static
cd frontend
pnpm install
pnpm add -D @sveltejs/adapter-static tailwindcss @tailwindcss/vite
```

### Configuration

svelte.config.js:
- Use adapter-static with fallback: 'index.html' (SPA mode)

vite.config.ts:
- Add tailwindcss vite plugin
- Proxy /api to http://localhost:4174 during dev

src/app.css:
- @import "tailwindcss"

### Pages to Create

1. `src/routes/+layout.svelte` — Root layout with sidebar navigation:
   - Logo: "Codeilus" text
   - Nav items: Learn, Explore, Ask, Settings
   - Main content area
   - Dark background, clean design

2. `src/routes/+page.svelte` — Welcome/home page:
   - "Welcome to Codeilus" hero
   - Brief description
   - "Start Learning" CTA button (links to /learn)
   - Status indicators (files parsed, symbols found, etc.)

3. `src/routes/learn/+page.svelte` — Learning page (placeholder):
   - "Learning Path" header
   - "Analysis required" message if no data
   - Will be filled with chapter grid in Sprint 6

4. `src/routes/explore/+page.svelte` — Explore landing:
   - Cards linking to: Tree, Graph, Metrics, Diagrams

5. `src/routes/explore/tree/+page.svelte` — File tree (placeholder):
   - Will be filled with file tree in Sprint 1 Agent 1A

6. `src/routes/ask/+page.svelte` — Q&A page (placeholder):
   - Chat interface placeholder
   - Will be filled with streaming Q&A in Sprint 5

### Stores

`src/lib/stores/websocket.ts`:
```typescript
// WebSocket connection to /api/v1/ws
// Auto-reconnect on disconnect
// Exposes: events (readable store of latest events)
```

`src/lib/stores/api.ts`:
```typescript
// Fetch wrapper for /api/v1/* endpoints
// Typed responses
export async function fetchHealth(): Promise<{ status: string }>;
```

### Build Verification
```bash
cd frontend && pnpm build
# Should produce frontend/build/ with index.html
cd .. && cargo build
# rust-embed should pick up frontend/build/
```

### Design System
- Dark mode default (gray-900 background, gray-100 text)
- Accent color: indigo-500
- Monospace font for code
- Sidebar: 240px fixed width
- Responsive: sidebar collapses to hamburger on mobile

### Rules
- Only create/modify files inside frontend/
- Use Svelte 5 runes ($state, $derived, $effect) not Svelte 4 stores
- Use TypeScript everywhere
- adapter-static with SPA fallback (all routes pre-render to index.html)
- Run `pnpm build` before finishing to verify it compiles
```

---

## Wave 2: Sprint 2 — Graph + API Routes (2 agents, after Wave 1)

### Agent 2A: codeilus-graph (Knowledge Graph Builder)

```
You are implementing codeilus-graph — the knowledge graph builder.

## Context
Read CLAUDE.md and NORTH_STAR.md. Read ../GitNexus/src/core/ingestion/ for graph patterns.

## Prerequisites (from Wave 1)
- codeilus-parse produces ParsedFile with symbols, imports, calls, heritage
- codeilus-db has FileRepo, SymbolRepo, EdgeRepo

## Cargo.toml Dependencies
Add to crates/codeilus-graph/Cargo.toml:
- codeilus-core = { path = "../codeilus-core" }
- codeilus-db = { path = "../codeilus-db" }
- petgraph = "0.6"
- rayon = "1"
- tracing = { workspace = true }
- serde = { workspace = true }

## What to Build

1. `src/lib.rs` — pub fn build_graph(files: &[ParsedFile], db: &DbPool) -> CodeilusResult<GraphStats>
2. `src/call_graph.rs` — Match calls to symbol definitions, assign confidence scores
3. `src/dependency.rs` — Build file-level dependency edges from resolved imports
4. `src/heritage.rs` — Create EXTENDS/IMPLEMENTS edges from heritage data
5. `src/community.rs` — Louvain community detection using petgraph
6. `src/entry_points.rs` — Score and rank entry points (main, index, handlers, CLI)
7. `src/process.rs` — Detect execution flows: BFS from entry points through CALLS edges
8. `src/types.rs` — GraphStats, CommunityInfo, ProcessInfo structs

### Louvain Community Detection
- Build undirected graph from edges (both CALLS and IMPORTS)
- Implement Louvain modularity optimization
- Output: community assignments (which symbols belong to which community)
- Store in communities + community_members tables

### Confidence Scoring
- Direct name match in same file: 1.0
- Name match in imported module: 0.8
- Name match elsewhere in project: 0.5
- Partial match / common name: 0.3

### Tests
- Create small test graphs and verify community detection
- Test call matching with various confidence levels
- Test entry point scoring

### Rules
- Only modify files inside crates/codeilus-graph/
- Store results via EdgeRepo, CommunityRepo in codeilus-db
- Use petgraph for in-memory graph algorithms
- Run `cargo test -p codeilus-graph && cargo clippy -p codeilus-graph`
```

### Agent 2B: API Routes for Files, Symbols, Graph

```
You are adding API routes to codeilus-api for files, symbols, and graph data.

## Context
Read CLAUDE.md. Read crates/codeilus-api/src/ to understand the existing Axum setup (lib.rs, state.rs, routes/).

## Prerequisites (from Wave 1)
- codeilus-db has FileRepo, SymbolRepo with CRUD methods
- AppState currently has db: Arc<DbPool> and event_bus: Arc<EventBus>

## What to Build

### Update AppState (src/state.rs)
Keep it simple — repos are created from DbPool on demand, not stored in state.

### New Route Files

1. `src/routes/files.rs`:
   - GET /files — list all files (with optional ?language= filter)
   - GET /files/:id — get single file
   - GET /files/:id/symbols — list symbols for a file

2. `src/routes/symbols.rs`:
   - GET /symbols — list all symbols (with optional ?kind= filter)
   - GET /symbols/:id — get single symbol
   - GET /symbols/search?q=name — search by name

3. `src/routes/graph.rs` (stub for Sprint 2):
   - GET /graph — return edges with source/target symbols
   - GET /communities — list communities
   - GET /processes — list execution flows

### Update routes/mod.rs
Add the new route modules and merge them into the router.

### Update codeilus-api Cargo.toml
Add dependency on codeilus-db (already there) — make sure FileRepo/SymbolRepo are accessible.

### Response Format
All endpoints return JSON. Use serde Serialize on the row types from codeilus-db.

### Tests
- Use tower::ServiceExt::oneshot to test each endpoint
- Test with in-memory DB + migrator setup

### Rules
- Only modify files inside crates/codeilus-api/
- Follow the existing pattern in routes/health.rs
- Run `cargo test -p codeilus-api && cargo clippy -p codeilus-api`
```

---

## Wave 3: Sprint 3+4 — Metrics + Diagrams (3 agents, after Wave 2)

### Agent 3A: codeilus-metrics

```
You are implementing codeilus-metrics — code metrics computation.

## Context
Read CLAUDE.md and NORTH_STAR.md. Read ../emerge/emerge/metrics/ for metric patterns.

## What to Build
1. `src/lib.rs` — pub fn compute_metrics(db: &DbPool) -> CodeilusResult<MetricsStats>
2. `src/sloc.rs` — Lines of code per file (from ParsedFile or re-count)
3. `src/fan.rs` — Fan-in (how many call this), fan-out (how many this calls) per symbol
4. `src/complexity.rs` — Cyclomatic complexity estimate from symbol line count + edge count
5. `src/modularity.rs` — Louvain modularity score (Q-score from community detection)
6. `src/tfidf.rs` — TF-IDF keywords per community (using symbol/file names)
7. `src/heatmap.rs` — Combined hotspot score: complexity * churn * fan_in
8. `src/types.rs` — FileMetrics, MetricsStats structs

Store results in file_metrics table via new FileMetricsRepo in codeilus-db.

### Rules
- Only modify: crates/codeilus-metrics/ and crates/codeilus-db/src/repos/ (add file_metrics_repo.rs)
- Run `cargo test -p codeilus-metrics && cargo clippy -p codeilus-metrics`
```

### Agent 3B: codeilus-analyze

```
You are implementing codeilus-analyze — anti-pattern and code quality detection.

## Context
Read CLAUDE.md and NORTH_STAR.md. Read ../GitVizz/gitvizz/graph_search_tool.py for anti-pattern patterns.

## What to Build
1. `src/lib.rs` — pub fn analyze(db: &DbPool) -> CodeilusResult<Vec<Pattern>>
2. `src/god_class.rs` — Detect classes with >20 methods or >500 LOC
3. `src/long_method.rs` — Detect methods with >50 LOC
4. `src/circular_deps.rs` — DFS cycle detection in import graph (using petgraph)
5. `src/security.rs` — Heuristic hotspot detection (eval, exec, SQL strings, hardcoded secrets patterns)
6. `src/test_gaps.rs` — Files with high complexity but no corresponding test file
7. `src/types.rs` — Pattern { kind, severity, file_id, symbol_id, description }

Store results in patterns table via new PatternRepo in codeilus-db.

### Rules
- Only modify: crates/codeilus-analyze/ and crates/codeilus-db/src/repos/ (add pattern_repo.rs)
- Severity levels: "info", "warning", "error"
- Run `cargo test -p codeilus-analyze && cargo clippy -p codeilus-analyze`
```

### Agent 3C: codeilus-diagram

```
You are implementing codeilus-diagram — Mermaid diagram generation.

## Context
Read CLAUDE.md and NORTH_STAR.md. Read ../CodeVisualizer/src/ir/ for FlowchartIR. Read ../gitdiagram/backend/app/prompts.py for LLM diagram pipeline.

## What to Build
1. `src/lib.rs` — pub fn generate_architecture(db: &DbPool) -> CodeilusResult<String> (Mermaid syntax)
2. `src/architecture.rs` — Communities → Mermaid subgraphs with inter-community edges
3. `src/flowchart.rs` — FlowchartIR: AST-like nodes (Entry, Exit, Decision, Process, Loop) → Mermaid
4. `src/file_tree.rs` — ASCII file tree generation (4 styles from GitHubTree)
5. `src/mermaid.rs` — Mermaid syntax helpers (escape labels, validate syntax)
6. `src/types.rs` — FlowchartIR, FlowchartNode, FlowchartEdge

### Rules
- Only modify: crates/codeilus-diagram/
- Mermaid output must be valid syntax (test by checking for balanced brackets, valid node IDs)
- Run `cargo test -p codeilus-diagram && cargo clippy -p codeilus-diagram`
```

---

## Wave 4: Sprint 5+6 — LLM + Learning (3 agents, after Wave 3)

### Agent 4A: codeilus-llm (Claude Code Integration)

```
You are implementing codeilus-llm — Claude Code CLI subprocess integration.

## Context
Read CLAUDE.md. Read ../forge-project/crates/forge-process/src/spawn.rs and parse.rs for the Claude CLI spawning pattern.

## What to Build
1. `src/lib.rs` — pub async fn ask(prompt: &str, context: &str) -> CodeilusResult<String>
2. `src/spawn.rs` — Spawn `claude` CLI with `--output-format stream-json --print` flags
3. `src/parse.rs` — Parse stream-json output (content_block_delta events → concatenate text)
4. `src/context.rs` — Build context string from graph data (<8K tokens: relevant symbols, edges, file snippets)
5. `src/types.rs` — LlmResponse, StreamEvent structs

### Key: stream-json format
Claude CLI outputs newline-delimited JSON. Each line has a "type" field:
- "content_block_start" → new content block beginning
- "content_block_delta" → { "delta": { "text": "..." } } → accumulate these
- "content_block_stop" → block done
- "message_stop" → all done

### Rules
- Only modify: crates/codeilus-llm/
- Graceful degradation: if `claude` binary not found, return Err(CodeilusError::Llm("Claude Code not found"))
- Run `cargo test -p codeilus-llm && cargo clippy -p codeilus-llm`
```

### Agent 4B: codeilus-narrate (Narrative Generator)

```
You are implementing codeilus-narrate — pre-generates all narrative content using Claude Code.

## Context
Read CLAUDE.md and NORTH_STAR.md (section 6.4). Read ../PocketFlow-Tutorial-Codebase-Knowledge/ for chapter writing prompts.

## Prerequisites
- codeilus-llm provides: ask(prompt, context) -> String
- codeilus-db has NarrativeRepo for storage

## What to Build
1. `src/lib.rs` — pub async fn narrate_all(db: &DbPool, llm: &LlmClient) -> CodeilusResult<NarrateStats>
2. `src/overview.rs` — Generate 30-second overview narrative
3. `src/architecture.rs` — Generate architecture-in-English narrative
4. `src/reading_order.rs` — Generate ranked reading order (key files first)
5. `src/extension.rs` — Generate "how to extend" guide
6. `src/contribution.rs` — Generate "how to contribute" guide
7. `src/trending.rs` — Generate "why it's trending" narrative
8. `src/community_summary.rs` — Generate per-community summary
9. `src/prompts.rs` — All prompt templates (adapt from PocketFlow's prompts)
10. `src/types.rs` — NarrateStats, NarrativeRequest structs

### Prompt Strategy
Each narrative type has a prompt template that receives structured context from the graph:
- Overview: file count, language breakdown, top-level structure, README excerpt
- Architecture: community names, inter-community edges, entry points
- Reading order: entry point scores, fan-in rankings, community centrality

### Rules
- Only modify: crates/codeilus-narrate/ and crates/codeilus-db/src/repos/narrative_repo.rs
- Store all results in narratives table with (kind, target_id, content)
- Run `cargo test -p codeilus-narrate && cargo clippy -p codeilus-narrate`
```

### Agent 4C: codeilus-learn (Learning Engine)

```
You are implementing codeilus-learn — the gamified learning engine.

## Context
Read CLAUDE.md and NORTH_STAR.md (section 6.5). Read ../PocketFlow-Tutorial-Codebase-Knowledge/ for pedagogical ordering patterns.

## Prerequisites
- Communities exist in DB (from graph)
- Narratives exist in DB (from narrate)
- Metrics exist in DB (from metrics)

## What to Build
1. `src/lib.rs` — pub fn generate_curriculum(db: &DbPool) -> CodeilusResult<CurriculumStats>
2. `src/curriculum.rs` — Topological sort communities, generate chapter order
3. `src/chapter.rs` — Chapter struct with sections, difficulty, community link
4. `src/progress.rs` — Track completion per section/chapter, calculate percentages
5. `src/gamification.rs` — XP calculation, badge awarding logic, streak tracking
6. `src/quiz.rs` — Generate quiz questions from graph data (multiple choice, true/false)
7. `src/types.rs` — Chapter, Section, Progress, Badge, LearnerStats, QuizQuestion

### Chapter Generation Rules
1. Chapter 0: "The Big Picture" — overview + architecture diagram
2. Chapters 1-N: one per community, topologically sorted (dependencies first)
3. Final chapter: "Putting It All Together" — cross-cutting flows
4. Within each chapter: files ordered by dependency (imported before importing)
5. Difficulty: beginner (<10 complexity avg), intermediate (10-30), advanced (>30)

### Gamification
- XP: +10 section, +50 chapter, +25 quiz, +5 graph explore, +5 Q&A
- 8 badges: First Steps, Chapter Champion, Graph Explorer, Quiz Master, Deep Diver, Completionist, Polyglot, Code Detective
- Streak: consecutive days with any activity

### Rules
- Only modify: crates/codeilus-learn/
- Store chapters, progress, badges, quizzes in respective DB tables
- Add repos to codeilus-db as needed (chapter_repo.rs, progress_repo.rs, etc.)
- Run `cargo test -p codeilus-learn && cargo clippy -p codeilus-learn`
```

---

## Wave 5: Sprint 7 — Harvest + Export (2 agents, after Wave 4)

### Agent 5A: codeilus-harvest (GitHub Trending Scraper)

```
You are implementing codeilus-harvest — GitHub trending scraper and clone manager.

## Context
Read CLAUDE.md and NORTH_STAR.md (Sprint 7 section).

## What to Build
1. `src/lib.rs` — pub async fn harvest_trending(date: &str, db: &DbPool) -> CodeilusResult<HarvestStats>
2. `src/scraper.rs` — Parse github.com/trending HTML (use reqwest + scraper crate)
3. `src/cloner.rs` — git clone --depth=1 with concurrency limit (5 parallel, use tokio::sync::Semaphore)
4. `src/fingerprint.rs` — Check harvested_repos table, skip already-analyzed
5. `src/types.rs` — TrendingRepo, HarvestStats

### Cargo.toml Dependencies
- codeilus-core, reqwest (with "rustls-tls"), scraper, tokio, tracing, serde

### Rules
- Only modify: crates/codeilus-harvest/
- Store results in harvested_repos table
- Run `cargo test -p codeilus-harvest && cargo clippy -p codeilus-harvest`
```

### Agent 5B: codeilus-export (Static HTML Generator)

```
You are implementing codeilus-export — generates self-contained HTML pages from analyzed repos.

## Context
Read CLAUDE.md and NORTH_STAR.md (Sprint 7, "Static Page Information Hierarchy").

## What to Build
1. `src/lib.rs` — pub fn export_repo(db: &DbPool, output: &Path) -> CodeilusResult<ExportStats>
2. `src/renderer.rs` — Build HTML string from template + data
3. `src/template.rs` — HTML template with inline CSS + vanilla JS
4. `src/data.rs` — Collect all data from DB into ExportData struct (JSON-serializable)
5. `src/index.rs` — Generate daily index page listing all repos
6. `src/types.rs` — ExportData, ExportStats

### Page Structure (all inlined in single HTML file)
- Hero: repo name, purpose, stars, language badges
- 30-Second Overview (from narratives)
- Architecture Diagram (Mermaid rendered to SVG, or Mermaid JS loaded inline)
- Key Files to Read First
- Entry Points
- How It Works
- How to Extend
- How to Contribute
- Metrics Snapshot
- Deep Dive (collapsible, lazy-loaded data)

### Target: <500KB per page, loads in <1s, no external dependencies

### Rules
- Only modify: crates/codeilus-export/ and export-template/
- All CSS inline, all JS inline, all data as JSON in <script> tags
- Run `cargo test -p codeilus-export && cargo clippy -p codeilus-export`
```

---

## Wave 6: Sprint 8 — MCP + Integration (1 agent, after Wave 5)

### Agent 6A: codeilus-mcp + Integration Wiring

```
You are implementing codeilus-mcp and wiring the full analysis pipeline in codeilus-app.

## Context
Read CLAUDE.md. Read ../forge-project/crates/forge-mcp-bin/ for the MCP server pattern.

## What to Build

### codeilus-mcp (8 MCP tools)
1. query_symbols — search symbols by name/kind
2. query_graph — get edges for a symbol
3. get_context — build context string for a file/symbol
4. get_impact — blast radius analysis for a symbol
5. get_diagram — return architecture/flowchart Mermaid
6. get_metrics — return metrics for a file
7. get_learning_status — return progress/chapters
8. explain_symbol — return/generate symbol explanation

### Pipeline Wiring (codeilus-app/src/main.rs)
Wire the full `analyze` command:
1. Parse repository (codeilus-parse)
2. Store files + symbols (codeilus-db)
3. Build graph (codeilus-graph)
4. Compute metrics (codeilus-metrics)
5. Run analysis (codeilus-analyze)
6. Generate diagrams (codeilus-diagram)
7. Generate narratives (codeilus-narrate)
8. Generate curriculum (codeilus-learn)
9. Start server (codeilus-api)

### Rules
- Only modify: crates/codeilus-mcp/ and crates/codeilus-app/src/main.rs
- Add necessary dependencies to Cargo.toml files
- Run `cargo build && cargo test && cargo clippy`
```

---

## Conflict Prevention Rules

1. **Each agent owns specific crate directories** — never touch another agent's crates
2. **codeilus-core is READ-ONLY** after Sprint 0 — if you need a new shared type, document it and we'll add it between waves
3. **codeilus-db/src/repos/** is shared — agents adding repos must use unique file names (file_repo.rs, symbol_repo.rs, etc.)
4. **Cargo.toml edits** — each agent only edits their own crate's Cargo.toml, never the workspace root
5. **migrations/** — no new migrations during parallel work. Schema changes go through a coordination step between waves.

## Verification Between Waves

After each wave completes:
```bash
cargo build          # all crates compile together
cargo test           # all tests pass
cargo clippy         # zero warnings
```

If there are conflicts in shared files (repos/mod.rs, api routes/mod.rs), resolve them manually before starting the next wave.
