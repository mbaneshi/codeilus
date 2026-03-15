# Codeilus — Inspection Report (2026-03-15)

## Architecture Inspection

### Crate Dependency Graph (verified from Cargo.toml)

```
codeilus-app (binary)
├── codeilus-api
│   ├── codeilus-core       (IDs, errors, events)
│   ├── codeilus-db          (SQLite, repos)
│   ├── codeilus-parse       (tree-sitter)
│   ├── codeilus-graph       (petgraph knowledge graph)
│   ├── codeilus-metrics     (code metrics)
│   ├── codeilus-analyze     (anti-patterns)
│   ├── codeilus-diagram     (mermaid/flowchart)
│   ├── codeilus-search      (BM25 FTS5)
│   ├── codeilus-llm         (Claude CLI)
│   ├── codeilus-narrate     (narrative generation)
│   └── codeilus-learn       (curriculum/quizzes)
├── codeilus-harvest         (GitHub trending)
├── codeilus-export          (static HTML)
└── codeilus-mcp             (MCP stdio server)
```

**Rule compliance**: core has zero internal deps. db depends only on core. No cross-sibling deps. All IDs are i64 newtypes.

### Data Flow

```
Filesystem → codeilus-parse → ParsedFile[]
    → codeilus-graph → KnowledgeGraph (petgraph DiGraph)
    → codeilus-metrics → FileMetrics[]
    → codeilus-analyze → Pattern[] (anti-patterns, security)
    → codeilus-diagram → Mermaid strings
    → codeilus-narrate → Narrative[] (LLM or placeholder)
    → codeilus-learn → Chapter[] (curriculum)
    → codeilus-db → SQLite (persistent)
    → codeilus-api → Axum HTTP → SvelteKit frontend
    → codeilus-mcp → stdio MCP → AI agents
```

---

## Graph Quality Inspection

### Edge Resolution Analysis

| Source | How edges are created | Count | Issue |
|--------|----------------------|-------|-------|
| `call_graph.rs` | Match callee name against symbol index | 2,537 | Working well |
| `dep_graph.rs` | File-level imports → first symbol per file | 2 | Almost no matches — import resolution too strict |
| `heritage.rs` | extends/implements from tree-sitter | 1 | Only 1 heritage edge in entire codebase |

**Root cause of sparse imports**: `dep_graph::build_dep_edges` resolves imports by exact file path match. Since parsed imports are like `crate::types::GraphNode` but file_index keys are filesystem paths like `./crates/codeilus-graph/src/types.rs`, they rarely match.

**Root cause of sparse heritage**: Rust doesn't have classical inheritance. The heritage queries look for `extends`/`implements` which only appear in TypeScript/Java. The single edge is likely from a TypeScript file.

### Community Detection Inspection

The Louvain algorithm produces reasonable clusters for connected nodes but:
- 209/875 symbols are completely isolated (no edges) → all go into one "misc" community
- Labels derived from parent directory name → all say "src"
- Cohesion scores are low (many communities have 0.0 cohesion)

### Entry Point Scoring

After fix: 30 entry points with scores >= 0.5, capped at 30. Scoring:
- `main` → 1.0 (name match) + 0.5 (zero callers + calls others) = 1.5
- Handlers/routes → 0.7 + structural bonuses
- Zero-caller symbols that call others → 0.5

---

## Frontend Inspection

### Pages Implemented

| Route | Status | Notes |
|-------|--------|-------|
| `/` | Working | Welcome page with stats |
| `/explore` | Working | Hub with links to tree/graph/metrics |
| `/explore/tree` | Working | File tree + symbol sidebar + source viewer |
| `/explore/graph` | Working | 3D WebGL force-directed graph |
| `/explore/metrics` | Basic | Metrics dashboard |
| `/learn` | Basic | Chapter grid with XP/badges |
| `/learn/[id]` | Basic | Chapter detail with tabs |
| `/ask` | Placeholder | Q&A chat (needs LLM) |

### Frontend Tech

- SvelteKit 5 with runes ($state, $derived, $effect)
- TailwindCSS 4 with CSS custom properties for theming
- 3d-force-graph for WebGL graph (dynamically imported, SSR-safe)
- adapter-static for compile-time embedding into Rust binary
- lucide-svelte for icons

---

## API Inspection

### Endpoints (from routes/*.rs)

| Method | Path | Status |
|--------|------|--------|
| GET | `/api/v1/health` | Working |
| GET | `/api/v1/ws` | Working (WebSocket events) |
| GET | `/api/v1/files` | Working |
| GET | `/api/v1/files/:id` | Working |
| GET | `/api/v1/files/:id/symbols` | Working |
| GET | `/api/v1/files/:id/source` | Working (needs repo_root) |
| GET | `/api/v1/symbols` | Working |
| GET | `/api/v1/symbols/:id` | Working |
| GET | `/api/v1/graph` | Working |
| GET | `/api/v1/communities` | Working |
| GET | `/api/v1/processes` | Working |
| GET | `/api/v1/metrics` | Working |
| GET | `/api/v1/patterns` | Working |
| GET | `/api/v1/learn/path` | Working |
| GET | `/api/v1/learn/chapter/:id` | Working |
| POST | `/api/v1/search` | Basic |

---

## Test Coverage Inspection

243 tests across all crates:

| Crate | Tests | Coverage |
|-------|-------|----------|
| codeilus-analyze | 16 | Good — all detectors tested |
| codeilus-api | 13 | Good — route integration tests |
| codeilus-graph | 17 | Good — all algorithms tested |
| codeilus-db | ~10 | Good — repo CRUD tests |
| codeilus-parse | ~15 | Good — per-language parsing |
| codeilus-metrics | ~8 | Basic |
| codeilus-search | ~5 | Basic |
| Others | Various | Minimal |

---

## Known Issues

1. **Import edge resolution** — dep_graph matches by exact file path, misses most Rust `use` imports
2. **Community labels** — all "src", not meaningful
3. **Curriculum build is slow** — 60s for curriculum generation (line: "Curriculum built and persisted chapters=232")
4. **232 chapters** — one per community, most are noise from the misc bucket
5. **LLM integration fragile** — Claude CLI subprocess with timeout/retry, no proper error recovery
6. **No incremental analysis** — full re-parse on every run
7. **Frontend not visually verified** — 3D graph and source viewer implemented but not browser-tested
