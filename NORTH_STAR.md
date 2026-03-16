# Codeilus — North Star Document

> Turn any codebase into an interactive learning experience.

---

## 1. Purpose & Vision

AI coding tools have enhanced developer productivity 100x. But **human understanding hasn't kept pace**. Developers ship faster than ever, yet struggle to deeply understand the codebases they work in — especially unfamiliar ones.

**Codeilus** bridges this gap. It's a single Rust binary that:
1. Extracts *everything* from any codebase (structure, relationships, metrics, patterns)
2. Presents it as a **structured, gamified, interactive learning experience** in the browser
3. Optionally publishes **static "grasp pages"** for trending repos — understand any project in 5 minutes

**The only prerequisite:** Claude Code available in terminal.

### Why This Matters

| Before Codeilus | After Codeilus |
|---|---|
| Grep around, hope you find the right file | Guided reading order: "read these 3 files, understand 80%" |
| Stare at a class and guess what it does | LLM-generated explanations with graph context |
| No idea how modules connect | Interactive architecture diagram from real data |
| Onboarding takes weeks | Gamified learning path: chapters, quizzes, progress |
| README is the only overview | 8 types of pre-generated narrative content |
| Understanding dies with the author | Published grasp pages for every trending repo, daily |

---

## 2. Two Modes, One Engine

### Mode 1: Interactive Local Server

```
$ codeilus ./any-repo
Analyzing... 342 files, 2847 symbols, 12 communities
Narrating... 8 sections generated
Open http://localhost:4174 to start learning
```

Full SvelteKit 5 UI with:
- Guided learning path (chapters ordered by dependency)
- Interactive graph explorer (nodes colored by community)
- Metrics dashboard (complexity heatmap, fan-in/out, hotspots)
- Architecture & flowchart diagrams (Mermaid, auto-generated from real graph)
- Streaming Q&A powered by Claude Code
- Gamification: XP, badges, streaks, quizzes
- Code search with BM25 ranking
- Impact analysis (blast radius of any symbol)

### Mode 2: Static Page Publishing

```
$ codeilus harvest --trending --date 2026-03-12
Scraping GitHub trending... 25 repos found
Analyzing tambo-ai/tambo... done
Narrating... Exporting...
Published: ./output/2026-03-12/tambo-ai-tambo.html (287KB)

$ codeilus deploy ./output --cloudflare
Deployed to codeilus.pages.dev/2026-03-12/
```

Self-contained HTML pages (~200-500KB each) that load in <1s on mobile:
- Daily automated via GitHub Actions cron
- No server needed — pure static files on CDN
- "5-minute grasp" for every trending repo

---

## 3. Use Cases

### UC-1: New Team Member Onboarding
**Actor:** Developer joining a team
**Flow:** `codeilus ./company-monorepo` → browser → Chapter 1: Big Picture → guided reading order → complete chapters → earn badges → ask questions via Q&A
**Value:** Weeks of onboarding compressed to hours. Structured path instead of random exploration.

### UC-2: Open Source Exploration
**Actor:** Developer evaluating a library
**Flow:** `codeilus ./interesting-lib` → architecture diagram → entry points → "How to extend" guide → metrics snapshot
**Value:** Understand architecture and extension points before committing to a dependency.

### UC-3: Code Review Preparation
**Actor:** Senior developer reviewing unfamiliar module
**Flow:** `codeilus ./repo` → navigate to module → see community graph → check complexity metrics → identify anti-patterns → understand blast radius
**Value:** Informed reviews with data, not gut feeling.

### UC-4: Daily Trending Discovery
**Actor:** Developer browsing codeilus.dev
**Flow:** Visit daily digest → click interesting repo → read 30-second overview → view architecture → check key files → expand deep dive
**Value:** Understand any trending project in 5 minutes without cloning.

### UC-5: AI Agent Integration
**Actor:** Claude Code / Cursor / other AI tool
**Flow:** MCP server provides 8 tools → agent queries graph, gets context, understands impact → better code suggestions
**Value:** AI tools get structured codebase understanding, not just grep results.

### UC-6: Teaching & Workshops
**Actor:** Instructor explaining a codebase
**Flow:** `codeilus ./example-project` → project to screen → walk through chapters in order → students complete quizzes → track class progress
**Value:** Structured curriculum auto-generated from any codebase.

---

## 4. Architecture Overview

### Single Binary, Everything Embedded

```
codeilus (single binary, ~15MB)
├── Rust backend (Axum HTTP + WebSocket)
├── SQLite database (WAL mode, zero config)
├── SvelteKit 5 frontend (embedded via rust-embed)
└── Claude Code integration (subprocess, stream-json)
```

No Docker. No PostgreSQL. No Redis. No Node.js runtime. Just one binary + Claude Code.

### Workspace: 16 Crates

```
codeilus/
├── Cargo.toml                     # workspace root
├── crates/
│   ├── codeilus-core/             # Events, errors, typed IDs, EventBus
│   ├── codeilus-parse/            # Tree-sitter 12-lang parsing + symbol extraction
│   ├── codeilus-graph/            # Knowledge graph: deps, calls, heritage, communities
│   ├── codeilus-metrics/          # SLOC, fan-in/out, complexity, modularity, heatmaps
│   ├── codeilus-analyze/          # Anti-patterns, security hotspots, suggestions
│   ├── codeilus-diagram/          # Mermaid/flowchart generation, LLM pipeline
│   ├── codeilus-search/           # BM25 full-text + RRF ranking
│   ├── codeilus-llm/              # Claude Code CLI spawn + stream-json parsing
│   ├── codeilus-narrate/          # Pre-generate all LLM content at analysis time
│   ├── codeilus-db/               # SQLite WAL, migrations, BatchWriter
│   ├── codeilus-api/              # Axum HTTP + WS + rust-embed frontend
│   ├── codeilus-learn/            # Curriculum, progress, gamification, quizzes
│   ├── codeilus-harvest/          # GitHub trending scraper, shallow clone queue
│   ├── codeilus-export/           # Static single-HTML renderer (vanilla JS)
│   ├── codeilus-mcp/              # MCP server (rmcp stdio, 8 tools)
│   └── codeilus-app/              # Binary: clap CLI, DB setup, server, shutdown
├── frontend/                      # SvelteKit 5 + adapter-static + TailwindCSS 4
├── export-template/               # Vanilla HTML/JS template for static export
└── migrations/                    # SQLite schema files
```

### Crate Dependency Graph

```
codeilus-app (binary)
├── codeilus-api
│   ├── codeilus-core
│   ├── codeilus-db
│   ├── codeilus-parse
│   ├── codeilus-graph
│   ├── codeilus-metrics
│   ├── codeilus-analyze
│   ├── codeilus-diagram
│   ├── codeilus-search
│   ├── codeilus-llm
│   ├── codeilus-narrate
│   └── codeilus-learn
├── codeilus-harvest
│   └── codeilus-core
├── codeilus-export
│   ├── codeilus-core
│   ├── codeilus-db
│   ├── codeilus-narrate
│   └── codeilus-diagram
├── codeilus-mcp
│   ├── codeilus-db
│   ├── codeilus-graph
│   └── codeilus-search
└── codeilus-db
    └── codeilus-core
```

---

## 5. Data Flow

### 5.1 Analysis Pipeline (runs once per repo)

```
                    ┌─────────────────────────────────────────────┐
                    │              codeilus analyze ./repo          │
                    └─────────────┬───────────────────────────────┘
                                  │
                    ┌─────────────▼───────────────┐
                    │   1. PARSE (codeilus-parse)  │
                    │                              │
                    │  Walk files (.gitignore)     │
                    │  Tree-sitter per language    │
                    │  Extract: symbols, imports,  │
                    │    calls, heritage            │
                    │  Parallel via rayon           │
                    │                              │
                    │  Output: ParsedFile[]        │
                    └─────────────┬───────────────┘
                                  │
                    ┌─────────────▼───────────────┐
                    │   2. GRAPH (codeilus-graph)  │
                    │                              │
                    │  Resolve imports → edges     │
                    │  Call tracing + confidence   │
                    │  Heritage (extends/impl)     │
                    │  Louvain community detection │
                    │  Execution flow detection    │
                    │  Entry point scoring          │
                    │                              │
                    │  Output: petgraph + DB edges │
                    └─────────────┬───────────────┘
                                  │
              ┌───────────────────┼───────────────────┐
              │                   │                   │
┌─────────────▼──────┐ ┌─────────▼────────┐ ┌───────▼─────────────┐
│ 3a. METRICS        │ │ 3b. ANALYZE      │ │ 3c. DIAGRAM         │
│ (codeilus-metrics)  │ │ (codeilus-analyze)│ │ (codeilus-diagram)   │
│                    │ │                  │ │                     │
│ SLOC, fan-in/out   │ │ God classes      │ │ Architecture →      │
│ Complexity         │ │ Long methods     │ │   Mermaid subgraphs │
│ Modularity score   │ │ Circular deps    │ │ Flowchart IR →      │
│ TF-IDF keywords    │ │ Security spots   │ │   Mermaid per fn    │
│ Git churn/contrib  │ │ Test gaps        │ │ ASCII file tree     │
│ Heatmap scoring    │ │ Suggestions      │ │ LLM auto-fix loop   │
└─────────────┬──────┘ └─────────┬────────┘ └───────┬─────────────┘
              │                   │                   │
              └───────────────────┼───────────────────┘
                                  │
                    ┌─────────────▼───────────────┐
                    │  4. NARRATE (codeilus-narrate)│
                    │                              │
                    │  Claude Code generates:       │
                    │  - 30-second overview         │
                    │  - Architecture in English    │
                    │  - Key files reading order    │
                    │  - How to extend              │
                    │  - How to contribute          │
                    │  - Why it's trending          │
                    │  - Per-community summaries    │
                    │  - Per-symbol explanations    │
                    │                              │
                    │  All stored in narratives DB │
                    └─────────────┬───────────────┘
                                  │
                    ┌─────────────▼───────────────┐
                    │  5. LEARN (codeilus-learn)   │
                    │                              │
                    │  Topological sort communities│
                    │  Generate chapters + order   │
                    │  Create quizzes from graph   │
                    │  Set difficulty from metrics  │
                    │  Chapter 0: Big Picture       │
                    │  Final: Putting It Together   │
                    └─────────────┬───────────────┘
                                  │
                    ┌─────────────▼───────────────┐
                    │  6. STORE (codeilus-db)      │
                    │                              │
                    │  20 SQLite tables            │
                    │  BatchWriter (50/2s flush)   │
                    │  Events persisted via bus    │
                    └─────────────────────────────┘
```

### 5.2 Interactive Server Flow

```
Browser ◄──── SvelteKit 5 (embedded) ────► Axum API ────► SQLite
   │                                          │
   └─── WebSocket ◄── EventBus (broadcast) ───┘
                          │
                    Claude Code CLI
                    (subprocess, stream-json)
```

### 5.3 Harvest → Export → Deploy Flow

```
GitHub Trending ──► codeilus-harvest ──► shallow clone queue
                                              │
                                    codeilus analyze (pipeline above)
                                              │
                                    codeilus-export ──► single HTML per repo
                                              │            (data inlined as JSON,
                                              │             Mermaid → inline SVG,
                                              │             CSS inlined, ~300KB)
                                              │
                                    codeilus deploy ──► Cloudflare Pages / gh-pages
                                              │
                                    codeilus.dev/YYYY-MM-DD/owner-repo
```

---

## 6. Core Modules Deep Dive

### 6.1 codeilus-core (the contract)

Every other crate depends on this. It defines:

| Type | Purpose |
|---|---|
| `EventBus` | tokio broadcast channel — all state changes flow through here |
| `EventSink` | Cheap cloneable handle for background tasks |
| `CodeilusEvent` | 18 variants (AnalysisStarted, ParsingProgress, GraphComplete, NarrativeGenerated, HarvestRepoFound, ExportComplete, Error, ...) |
| `CodeilusError` | 12 variants with thiserror (Database, Parse, Graph, Llm, Harvest, Export, Io, ...) |
| `FileId`, `SymbolId`, `EdgeId`, `CommunityId`, `ChapterId` | i64 newtype wrappers (SQLite rowid for fast graph joins) |
| `Language` | 13 variants (Python, TypeScript, JavaScript, Rust, Go, Java, C, Cpp, CSharp, Ruby, PHP, Swift, Kotlin) |
| `SymbolKind` | Function, Class, Method, Interface, Enum, Trait, Struct, Module, Constant, TypeAlias |
| `EdgeKind` | Calls, Imports, Extends, Implements, Contains |
| `Confidence(f64)` | 0.0 = guess, 1.0 = certain |
| `NarrativeKind` | Overview, Architecture, ExtensionGuide, ContributionGuide, WhyTrending, ModuleSummary, SymbolExplanation, ReadingOrder |

### 6.2 codeilus-parse (the extractor)

Tree-sitter integration for 12 languages. Each language has query strings for:
- **Definitions**: functions, classes, methods, interfaces, enums, traits, structs
- **Imports**: `import`, `use`, `require`, `include`
- **Calls**: function/method invocations
- **Heritage**: `extends`, `implements`, `impl ... for`

Key design:
- Filesystem walker respects `.gitignore` (via `ignore` crate)
- 20MB byte budget with chunked processing
- `rayon` parallel parsing (CPU-bound, not async)
- Import resolution is language-specific (tsconfig paths, Python relative, Go modules, Rust mod/use)

Output per file:
```rust
ParsedFile {
    path: String,
    language: Language,
    symbols: Vec<Symbol>,      // name, kind, line range, signature
    imports: Vec<Import>,      // source path, imported names
    calls: Vec<Call>,          // caller symbol, callee name, line
    heritage: Vec<Heritage>,   // child, parent, kind (extends/implements)
}
```

### 6.3 codeilus-graph (the brain)

Builds the knowledge graph from parsed data:

| Component | Algorithm | Output |
|---|---|---|
| Call graph | Symbol name matching + scope resolution | CALLS edges with confidence |
| Dependency graph | Resolved imports → file-level edges | IMPORTS edges |
| Heritage graph | extends/implements from parser | EXTENDS, IMPLEMENTS edges |
| Communities | Louvain on petgraph (undirected) | Community assignments + cohesion scores |
| Execution flows | BFS from entry points through CALLS edges | Process with ordered steps |
| Entry points | Heuristic scoring (main, index, handlers, CLI) | Ranked entry symbols |

### 6.4 codeilus-narrate (the storyteller)

Pre-generates ALL narrative content at analysis time using Claude Code:

| Narrative Kind | Input Context | Output |
|---|---|---|
| Overview | README + top-level structure + language stats | "What it does, for who, why it matters" (2-3 paragraphs) |
| Architecture | Community graph + inter-community edges + entry points | "How it's structured" (3-5 paragraphs) |
| ReadingOrder | Entry point scores + fan-in + community centrality | Ranked list: "Read these 3-5 files to understand 80%" |
| ExtensionGuide | High fan-in interfaces, plugin patterns, config points | "How to add features" (step-by-step) |
| ContributionGuide | Entry points, code patterns, test gaps | "How to contribute" (for newcomers) |
| WhyTrending | README + description + ecosystem context | "Why developers care" (1-2 paragraphs) |
| ModuleSummary | Per-community symbols + edges + metrics | "What this module does" (per community) |
| SymbolExplanation | Symbol signature + callers + callees + body | "What this function does" (per symbol, on-demand) |

All stored in the `narratives` table. Served instantly (no LLM latency) in both interactive and static modes.

### 6.5 codeilus-learn (the differentiator)

Transforms graph + narratives into a gamified curriculum:

**Curriculum generation:**
1. Topological sort communities by dependency (imported before importing)
2. Entry point communities first, core before features
3. Each community → one chapter
4. Chapter 0: "The Big Picture" (overview + architecture diagram)
5. Final chapter: "Putting It All Together" (cross-cutting flows)

**Chapter structure:**
- Title + overview (from narratives)
- Key concepts (TF-IDF keywords + descriptions)
- Interactive diagram (community subgraph)
- Code walkthrough (guided reading order)
- Connections to other chapters
- Quiz/challenge
- Difficulty rating (from complexity metrics)

**Gamification:**
| Action | XP |
|---|---|
| Complete a section | +10 |
| Complete a chapter | +50 |
| Pass a quiz | +25 |
| Explore graph | +5 |
| Ask a Q&A question | +5 |

| Badge | Requirement |
|---|---|
| First Steps | Complete Chapter 0 |
| Chapter Champion | Complete any chapter |
| Graph Explorer | Visit 10 different nodes |
| Quiz Master | Pass 5 quizzes |
| Deep Diver | Read 20 symbol explanations |
| Completionist | 100% progress |
| Polyglot | Explore files in 3+ languages |
| Code Detective | Find 3 anti-patterns |

### 6.6 codeilus-diagram (the visualizer)

Builds diagrams and flow representations from the graph and parsed code:

| Diagram | Method |
|---|---|
| Architecture | Communities → Mermaid subgraphs with inter-community edges |
| Function flowchart | AST → FlowchartIR (Entry, Exit, Decision, Process, Loop, Exception) → Mermaid |
| File tree | ASCII rendering, 4 styles (classic, slashed, minimal, compact) |
| LLM-enhanced diagrams | 3-stage pipeline: analyze → generate → validate/fix (up to 3 attempts), with auto-fix prompts for Mermaid syntax |
| **Smart labels (optional)** | *[from CodeVisualizer]* Optional LLM pass that rewrites terse node labels in flowcharts into human-friendly descriptions in the user’s language, without changing control-flow structure |

Smart labels are intentionally kept lightweight compared to full narratives: they improve readability of diagrams while `codeilus-narrate` and `codeilus-learn` own longer-form prose.

### 6.7 codeilus-export (the publisher)

Generates **single self-contained HTML per repo** and, optionally, a **PocketFlow-style markdown tutorial bundle**. Together these form Codeilus’s unified “wiki/tutorial export” story — a direct successor to GitNexus’s `wiki` output, but powered by Codeilus’s richer graph + learning engine.

```
hero: repo name, 1-line purpose, stars today, language badges
├── 30-Second Overview          (from narratives.overview)
├── Architecture Diagram        (Mermaid → inline SVG)
├── Key Files to Read First     (from narratives.reading_order)
├── Entry Points                (from graph entry point scoring)
├── How It Works                (from narratives.architecture)
├── How to Extend               (from narratives.extension_guide)
├── How to Contribute           (from narratives.contribution_guide)
├── Why It's Trending           (from narratives.why_trending)
├── Metrics Snapshot            (SLOC, complexity mini-heatmap)
└── Deep Dive [collapsible]     (full interactive graph/metrics, lazy-loaded)
```

Constraints:
- **200-500KB** per HTML page (all data inlined as JSON in `<script>` tags)
- **<1s load** on mobile
- **Zero server** needed — pure static HTML/JS/CSS
- **Vanilla JS** — no framework runtime
- **Markdown bundle** mirrors PocketFlow’s proven layout: `index.md` + numbered chapter files (e.g., `01_chapter_name.md`) with embedded diagrams and cross-references, so teams can treat it as a long-form “codebase wiki” checked into their own repos.

---

## 7. Database Schema

20 tables across 7 domains:

```
┌─────────── Core ───────────┐  ┌──────── Graph ────────┐
│ files (path, lang, sloc)   │  │ communities            │
│ symbols (name, kind, lines)│  │ community_members      │
│ edges (source→target, kind)│  │ processes              │
└────────────────────────────┘  │ process_steps          │
                                └────────────────────────┘

┌──────── Metrics ───────────┐  ┌──────── Narratives ────┐
│ file_metrics               │  │ narratives             │
│ patterns                   │  │  (kind, target, content)│
└────────────────────────────┘  └────────────────────────┘

┌──────── Learning ──────────┐  ┌──────── Harvest ───────┐
│ chapters                   │  │ harvested_repos        │
│ chapter_sections           │  │  (owner, name, status) │
│ progress                   │  └────────────────────────┘
│ quiz_questions             │
│ quiz_attempts              │  ┌──────── System ────────┐
│ badges                     │  │ events                 │
│ learner_stats              │  │ schema_version         │
└────────────────────────────┘  └────────────────────────┘
```

Key design decisions:
- **i64 IDs** (SQLite rowid) — not UUID — for fast graph joins
- **WAL mode** — concurrent reads during writes
- **BatchWriter** — events flush in batches of 50 or every 2 seconds
- **FTS5** planned for full-text search (Sprint 7)

---

## 8. Technology Choices

| Decision | Choice | Why |
|---|---|---|
| Language | Rust | Single binary, no runtime, memory safe, fast parsing |
| HTTP server | Axum 0.7 | Tokio-native, tower middleware, WebSocket built-in |
| Database | SQLite (rusqlite, WAL) | Zero config, single file, adequate for single-repo analysis |
| Frontend | SvelteKit 5 + adapter-static | Compiled to static files, embedded via rust-embed |
| CSS | TailwindCSS 4 | Utility-first, small bundle, dark mode built-in |
| Frontend embed | rust-embed | Baked into binary at compile time, no Node.js runtime |
| Parsing | tree-sitter | Battle-tested, incremental, 12+ language grammars |
| CPU parallelism | rayon | Tree-sitter parsing is CPU-bound, not I/O-bound |
| Async I/O | tokio | HTTP server, WebSocket, Claude CLI streaming |
| Graph algorithms | petgraph | In-memory for Louvain, BFS, cycle detection |
| LLM | Claude Code CLI subprocess | Zero API keys needed, stream-json output |
| Static export | Vanilla HTML/JS | Self-contained, no framework runtime, ~300KB |
| CDN deploy | Cloudflare Pages + GitHub Actions | Free tier sufficient, global CDN |
| MCP server | rmcp (Rust MCP SDK) | Official SDK, stdio transport |
| IDs | i64 newtype wrappers | Graph-heavy workload, faster joins than UUID |
| Community detection | Louvain (not Leiden) | Simpler Rust implementation, adequate quality |
| Narratives | Pre-generated at analysis time | Instant load for both interactive + static modes |

---

## 9. Sprint Roadmap

### Sprint 0: Foundation [DONE]
**Goal:** Skeleton compiles, serves empty frontend, has DB + EventBus.

| Deliverable | Status |
|---|---|
| 16-crate workspace compiles | Done |
| codeilus-core: EventBus, events, errors, IDs, types | Done |
| codeilus-db: DbPool (WAL), Migrator, BatchWriter, 20-table schema | Done |
| codeilus-api: Axum + rust-embed + SPA fallback + WebSocket | Done |
| codeilus-app: clap CLI with 6 subcommands | Done |
| 12 stub crates compile | Done |
| `cargo test` — 10 tests pass | Done |
| `cargo clippy` — zero warnings | Done |

**Acceptance:** `cargo build && ./target/debug/codeilus --help` shows all subcommands. `codeilus serve` starts HTTP server.

---

### Sprint 1: Parsing Engine
**Goal:** `codeilus analyze ./repo` extracts all symbols from 6 languages.

| Deliverable | Crate |
|---|---|
| Tree-sitter grammars: Python, TypeScript, JavaScript, Rust, Go, Java | codeilus-parse |
| Language detection from file extensions | codeilus-parse |
| Filesystem walker respecting .gitignore | codeilus-parse |
| Chunked parsing with rayon parallelism | codeilus-parse |
| Import resolution (language-specific) | codeilus-parse |
| FileRepo + SymbolRepo batch inserts | codeilus-db |
| API: GET /files, GET /files/:id/symbols, GET /symbols | codeilus-api |
| Frontend: file tree page with sortable hierarchy | frontend |
| CLI: `codeilus analyze ./path` stores results | codeilus-app |

**Acceptance:** `codeilus analyze ./some-python-repo && codeilus serve` → navigate to `/explore/tree` → see all files with symbols listed.

---

### Sprint 2: Knowledge Graph
**Goal:** Full graph with calls, deps, heritage, communities, execution flows.

| Deliverable | Crate |
|---|---|
| Call tracing with confidence scoring (0.0-1.0) | codeilus-graph |
| Heritage edges (extends/implements) | codeilus-graph |
| File dependency edges from resolved imports | codeilus-graph |
| Louvain community detection on petgraph | codeilus-graph |
| Execution flow detection (BFS from entry points) | codeilus-graph |
| Entry point scoring heuristics | codeilus-graph |
| EdgeRepo, CommunityRepo batch inserts | codeilus-db |
| API: GET /graph, GET /communities, GET /processes | codeilus-api |
| Frontend: interactive graph explorer (D3/Sigma.js) | frontend |

**Acceptance:** Navigate to `/explore/graph` → see interactive graph with colored communities. Click a node → see callers/callees in side panel.

---

### Sprint 3: Metrics & Analysis
**Goal:** All code metrics, anti-pattern detection, hotspot identification.

| Deliverable | Crate |
|---|---|
| SLOC, fan-in/out, cyclomatic complexity estimate | codeilus-metrics |
| Louvain modularity score, TF-IDF keywords per community | codeilus-metrics |
| Git metrics (churn, contributors via git2-rs) | codeilus-metrics |
| Heatmap scoring | codeilus-metrics |
| God class detection, long method detection | codeilus-analyze |
| Circular dependency detection (DFS cycle) | codeilus-analyze |
| Security hotspot heuristics | codeilus-analyze |
| Test coverage gap detection | codeilus-analyze |
| API: GET /metrics, GET /patterns, GET /hotspots | codeilus-api |
| Frontend: metrics dashboard with heatmap | frontend |

**Acceptance:** Navigate to `/explore/metrics` → see heatmap, pattern warnings highlight problematic files.

---

### Sprint 4: Diagrams & Flowcharts
**Goal:** Auto-generated architecture diagrams and function-level flowcharts.

| Deliverable | Crate |
|---|---|
| Architecture diagram: communities → Mermaid subgraphs | codeilus-diagram |
| FlowchartIR: AST → nodes/edges → Mermaid syntax | codeilus-diagram |
| LLM-enhanced diagram pipeline (optional) | codeilus-diagram |
| Mermaid validation + auto-fix loop (up to 3 attempts) | codeilus-diagram |
| ASCII file tree (4 styles) | codeilus-diagram |
| API: GET /diagram/architecture, GET /diagram/flowchart/:id | codeilus-api |
| Frontend: architecture page + flowchart page | frontend |

**Acceptance:** `/explore/architecture` shows auto-generated Mermaid diagram. `/explore/flowchart` → select function → see control flow.

---

### Sprint 5: LLM + Narrative Engine
**Goal:** Claude Code powers Q&A AND pre-generated narratives.

| Deliverable | Crate |
|---|---|
| Spawn `claude` subprocess with stream-json parsing | codeilus-llm |
| Context builder (relevant graph context, <8K tokens) | codeilus-llm |
| Pre-generate 8 narrative types at analysis time | codeilus-narrate |
| Store all narratives in DB | codeilus-narrate |
| Streaming Q&A: POST /ask | codeilus-api |
| Cached narratives: GET /narrative/:kind | codeilus-api |
| Frontend: Q&A chat, explain popover, narrative sections | frontend |
| **[from deep-research]** Streaming research UX: show tool usage steps as agent investigates | codeilus-api + frontend |
| **[from deep-research]** "Relevant files/symbols" sidebar alongside Q&A chat | frontend |
| **[from PocketFlow]** Per-prompt language instructions for multi-language narratives | codeilus-narrate |

**Graceful degradation:** Without Claude Code, all analysis/graphs/metrics/diagrams work. Narratives show placeholder text.

**Acceptance:** `codeilus analyze ./repo` generates narratives. `GET /api/v1/narrative/overview` returns instant pre-generated content. `POST /api/v1/ask` streams answers with visible tool-usage steps. Related files appear in sidebar during Q&A.

---

### Sprint 6: Learning Engine (THE DIFFERENTIATOR)
**Goal:** Gamified learning experience with chapters, quizzes, progress.

| Deliverable | Crate |
|---|---|
| Curriculum generator (topological sort, entry-first) | codeilus-learn |
| **[from PocketFlow]** Hybrid chapter ordering: topo-sort base + LLM pedagogical refinement | codeilus-learn |
| **[from PocketFlow]** LLM-powered community naming with beginner analogies | codeilus-learn |
| **[from PocketFlow]** Port chapter-writing prompts (analogies, <10 line snippets, cross-refs, Mermaid) | codeilus-learn |
| Chapter structure with sections, diagrams, walkthroughs | codeilus-learn |
| Progress tracking (per-section, per-chapter, overall %) | codeilus-learn |
| XP system + 8 badges | codeilus-learn |
| Streak tracking (consecutive days) | codeilus-learn |
| Quiz generator (multiple choice, true/false, impact Qs) | codeilus-learn |
| API: /learn/path, /learn/chapter/:id, /learn/progress, /learn/quiz, /learn/stats | codeilus-api |
| Frontend: chapter grid, progress bars, XP counter, badge shelf, quiz modal | frontend |

**Key insight:** PocketFlow is pure-LLM (guesses abstractions). We feed *real graph data* into PocketFlow-style prompts → better tutorials than either approach alone.

**Acceptance:** `/learn` shows chapter grid with beginner-friendly chapter names (not cluster IDs). Chapter 1 unlocked. Complete chapters → unlock next → earn badges. XP accumulates. Chapters cross-reference each other.

---

### Sprint 7: Harvest + Export + Deploy (THE PUBLISHING PIPELINE)
**Goal:** Automated daily trending → static pages → CDN, plus core search/impact plumbing for later MCP tools.

| Deliverable | Crate |
|---|---|
| GitHub trending scraper (configurable filters) | codeilus-harvest |
| Shallow clone queue with concurrency limit | codeilus-harvest |
| Repo fingerprinting (skip already-analyzed) | codeilus-harvest |
| Static single-HTML renderer (vanilla JS, data inlined) | codeilus-export |
| Daily index page + historical archive | codeilus-export |
| Deploy to Cloudflare Pages / gh-pages | codeilus-app |
| GitHub Actions daily workflow | .github/workflows |
| BM25 search via SQLite FTS5 (backing both UI search and MCP `query`) | codeilus-search |
| Impact analysis (blast radius, depth-scored risk, d=1/d=2/d=3 semantics) | codeilus-graph |
| Core change-impact engine (GitNexus-style `detect_changes` that maps diffs → changed symbols → affected processes) | codeilus-graph + codeilus-db |
| Core graph-aware rename engine (GitNexus-style `rename` that proposes multi-file edits with dry-run support) | codeilus-graph + codeilus-db |

**Acceptance:** `codeilus harvest --trending` finds repos. `codeilus export --all-harvested` generates HTML files <500KB each. `open output.html` loads in <1s. Internal crates expose reusable `detect_changes` and `rename` services that:
- Given a Git diff, return changed symbols and affected processes as structured data.
- Given a requested symbol rename, return a dry-run plan (per-file edits) and can apply it safely when invoked.

---

### Sprint 8: MCP & Polish
**Goal:** AI agent integration, incremental analysis, release.

| Deliverable | Crate |
|---|---|
| MCP stdio server with 8 tools aligned to GitNexus-style workflows but scoped to a **single analyzed repo**: `list_repos`/`current_repo`, `query`, `context`, `impact`, `detect_changes`, `rename`, `diagram`, `learn_status` | codeilus-mcp |
| `codeilus setup` auto-configure for Claude Code/Cursor | codeilus-app |
| Incremental analysis (only re-parse changed files) | codeilus-parse |
| Responsive frontend, dark/light theme, keyboard shortcuts | frontend |
| Cross-platform release binaries (macOS arm64/x64, Linux x64) | CI |
| Landing site: codeilus.dev with daily archive | frontend |

**Acceptance:** `codeilus mcp` starts MCP server. Claude Code can query the graph and call:
- `query` for BM25/RRF search.
- `context` for 360° symbol views.
- `impact` for blast radius with depth and confidence scoring.
- `detect_changes` to understand what a diff changed and which processes are affected.
- `rename` to perform graph-aware multi-file renames with dry-run previews.
`codeilus ./repo` on a previously analyzed repo only re-parses changed files.

---

## 10. Acceptance Criteria (End-to-End)

### Must Have (MVP)
- [x] `codeilus ./any-repo` → full analysis pipeline completes
- [x] Browser opens → guided learning path with chapters
- [x] Interactive graph explorer with colored communities
- [x] Metrics dashboard with complexity heatmap
- [x] Auto-generated architecture diagram (Mermaid)
- [x] Pre-generated narrative content (overview, architecture, reading order)
- [x] Streaming Q&A via Claude Code
- [x] Progress tracking (% complete per chapter)
- [x] XP + at least 4 badges
- [ ] Zero warnings (`cargo clippy`), all tests pass

### Should Have
- [x] `codeilus harvest --trending` → scrape, clone, analyze daily
- [x] `codeilus export` → self-contained HTML <500KB
- [ ] `codeilus deploy --cloudflare` → published to CDN
- [ ] Daily GitHub Actions automation
- [x] MCP server with 8 tools
- [x] Incremental analysis (re-parse only changed files)
- [ ] Function-level flowchart diagrams
- [x] Quiz system with adaptive difficulty

### Nice to Have
- [ ] Multi-language narrative translation (per-prompt strategy from PocketFlow, not post-translate)
- [ ] Search across all historical repos on codeilus.dev
- [ ] Dark/light theme toggle
- [ ] Keyboard shortcuts for power users
- [ ] Export to PDF
- [ ] Markdown export (numbered chapters + index, PocketFlow format)
- [ ] Semantic search alongside BM25 (from deep-research)
- [ ] Step-through execution trace visualization (from code-understanding-tools)

---

## 11. Testing Strategy

| Layer | Approach | Tool |
|---|---|---|
| Unit tests | Per crate (parsing, graph, metrics, curriculum, export) | `cargo test` |
| Integration tests | Analyze sample repo → verify DB → verify API → verify export | `cargo test --test integration` |
| Sample repos | 3 small repos in `tests/fixtures/` (Python, TypeScript, Rust) | Committed to repo |
| Export tests | Generated HTML is valid, <500KB, loads without server | Headless check |
| Harvest tests | Mock GitHub trending response → verify pipeline | Mock HTTP |
| Frontend | Component tests | Vitest |
| E2E smoke | analyze → serve → curl endpoints → export → validate | Shell script |
| Policy | Zero warnings, all tests pass | CI (`cargo check + clippy + test`) |

---

## 12. Reference Repos (10 total)

Codeilus combines the best ideas from these projects:

### Original 7 (analysis + infrastructure)

| Repo | What We Take |
|---|---|
| **GitNexus** | Chunked parsing pipeline, symbol extraction, import resolution, call tracing, confidence scoring, community detection, process detection |
| **emerge** | SLOC, fan-in/out, modularity, TF-IDF, git metrics, heatmap scoring |
| **GitVizz** | Anti-pattern detection, security hotspot heuristics |
| **CodeVisualizer** | FlowchartIR data structures, AST → IR conversion |
| **gitdiagram** | 3-stage LLM pipeline for diagrams, auto-fix prompts |
| **GitHubTree** | ASCII file tree styles, folder-first sorting |
| **forge-project** | Rust/Axum + Svelte 5 + SQLite WAL + rust-embed architecture, EventBus, BatchWriter, graceful shutdown |

### New 3 (narrative + UX + landscape)

| Repo | What We Take |
|---|---|
| **PocketFlow-Tutorial-Codebase-Knowledge** | Battle-tested chapter-writing prompts (analogies, <10 line snippets, Mermaid diagrams, cross-references), pedagogical chapter ordering (foundational → user-facing → implementation), LLM-driven community naming with beginner analogies, multi-language prompt strategy (per-prompt, not post-translate), static export format (numbered markdown chapters + index) |
| **deep-research** (codegen-sh) | Streaming research UX (show agent tool usage as it investigates), "relevant files" sidebar during Q&A, agent event streaming pattern (tool_start → tool_end → content events via SSE), semantic search concept |
| **code-understanding-tools** (cipher387) | Feature validation (confirms coverage), step-through execution visualization idea (future), code beautification for chapter walkthroughs |

### 12.1 What We Deliberately Do *Not* Port

To keep Codeilus focused and shippable, some capabilities from the reference repos are **explicit non-goals for v1**:

- **From GitNexus**
  - We **do not** adopt KuzuDB or a global multi-repo registry. Codeilus standardizes on **SQLite + petgraph** and a **single analyzed repo at a time**.
  - We **do not** depend on Node/TypeScript runtimes in the core engine. All backend logic is Rust; Node appears only in optional frontend tooling.
- **From GitVizz**
  - We **do not** copy the full SaaS/backend stack (FastAPI, MongoDB, Phoenix observability, multi-service infra).
  - We focus on the **analysis patterns** (anti-pattern detection, security hotspots, smart context ideas), implemented in Rust/SQLite.
- **From gitdiagram & deep-research**
  - We **do not** depend on Modal, serverless cloud runtimes, or a split Next.js + FastAPI architecture.
  - Their ideas (3-stage LLM diagram pipeline, streaming research UX, tool-usage events) are embedded into the **single Rust binary + Claude Code CLI** via Axum + WebSocket/SSE.
- **From PocketFlow-Tutorial-Codebase-Knowledge**
  - We **do not** reuse the pure-LLM architecture that guesses abstractions from raw code.
  - Instead, we treat PocketFlow as a **prompt and pedagogy library** layered on top of Codeilus’s static analysis, graph, and metrics.
- **From code-understanding-tools**
  - We **do not** attempt to replicate every external web tool.
  - We cherry-pick high-impact patterns for Codeilus (execution-trace visualization as a future feature, beautified snippets in exports) and leave the rest as inspiration, not requirements.

### Key Architectural Insight

PocketFlow is pure-LLM (guesses abstractions from raw code). Codeilus has rich static analysis. The combination is more powerful than either:

```
Combined: raw code ──tree-sitter──► graph ──LLM(with graph context)──► chapters
                                      ↑
                       communities, metrics, edges, entry points
                       = structured context for better prompts
```

The LLM doesn't guess abstractions — we tell it based on real data, and ask it to explain/name them for a human learner.

---

## 13. Current State

**Sprints 0–8: COMPLETE**

```
16 crates compile ✓    50+ REST API endpoints ✓    SSE streaming Q&A ✓

codeilus-core:     EventBus, 18 events, 12 errors, 5 ID types, 4 type enums
codeilus-db:       DbPool (WAL + r2d2 connection pool), Migrator, BatchWriter, Moka cache, 20-table schema
codeilus-parse:    Tree-sitter parsing for 12 languages, incremental parsing, pipeline checkpoints
codeilus-graph:    Knowledge graph with Louvain communities, pattern detection
codeilus-metrics:  File metrics (SLOC, complexity, fan-in/out)
codeilus-diagram:  Mermaid architecture diagrams
codeilus-llm:      LLM narratives (8 types via Claude Code CLI)
codeilus-narrate:  Pre-generated narrative content for all 8 types
codeilus-learn:    Curriculum generation with quizzes, gamification (XP, badges, streaks)
codeilus-api:      Axum + CORS + WS + rust-embed SPA fallback, 50+ endpoints, SSE streaming
codeilus-harvest:  GitHub trending harvest
codeilus-export:   Static HTML export
codeilus-mcp:      MCP server with 16 tools
codeilus-app:      clap CLI: analyze, serve, harvest, export, deploy, mcp
frontend:          SvelteKit 5 (graph explorer, learning path, Ask AI)
```

**Next:** Cloudflare deploy pipeline, GitHub Actions daily automation, cargo clippy zero-warning sweep.
