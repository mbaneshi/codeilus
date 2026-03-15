# Codeilus — Status Quo (2026-03-15)

## What Exists Today

Codeilus is a **single Rust binary** (16 crates, 25,846 lines of Rust + 17 frontend files) that:

1. **Parses** any codebase using tree-sitter (Rust, TypeScript, JavaScript, Python, Go, Java)
2. **Builds a knowledge graph** — symbols, call edges, imports, heritage, communities
3. **Computes metrics** — SLOC, complexity, fan-in/out, anti-patterns, security hotspots
4. **Serves a SvelteKit 5 frontend** with 3D graph explorer, file tree with source viewer, learning chapters
5. **Generates narratives** via Claude CLI (or placeholder fallback)
6. **Exposes an MCP server** for AI agent integration (8 tools)

### Build Health

| Check | Status |
|-------|--------|
| `cargo build` | Clean |
| `cargo test` | **243 tests passing** |
| `cargo clippy` | Zero warnings |
| Frontend build | SvelteKit 5 + TailwindCSS 4, embedded via rust-embed |

### Analysis Output (self-analysis of codeilus)

| Metric | Value |
|--------|-------|
| Files | 147 |
| Symbols | 875 |
| Edges | 2,540 |
| Communities | 23 |
| Entry Points | 30 |
| SLOC | 17,237 (Rust) + 759 (TypeScript) |
| Languages | Rust (136 files), TypeScript (9), Python (1), JavaScript (1) |

### Edge Distribution

| Kind | Count | Notes |
|------|-------|-------|
| CALLS | 2,537 | Call graph well-connected |
| IMPORTS | 2 | File-level dep promotion — needs improvement |
| EXTENDS | 1 | Heritage detection working but sparse |

### Community Health

23 communities (target was 5-15). Largest has 209 members (isolated nodes bucket).
Connected communities range from 2 to 130 members.
**Problem**: Labels are all "src" — the directory-based labeling picks parent dir, not meaningful names.

---

## 16 Crates — What Each Does

| Crate | Purpose | Maturity |
|-------|---------|----------|
| `codeilus-core` | IDs, errors, events, EventBus | Solid — foundational |
| `codeilus-db` | SQLite WAL, migrations, BatchWriter, repos | Solid |
| `codeilus-parse` | Tree-sitter 6-lang parsing + symbol extraction | Solid |
| `codeilus-graph` | Knowledge graph: calls, deps, heritage, communities, processes | Working, needs tuning |
| `codeilus-metrics` | SLOC, fan-in/out, complexity, modularity, TF-IDF | Working |
| `codeilus-analyze` | Anti-patterns, security, test gaps, suggestions | Working |
| `codeilus-diagram` | Mermaid architecture/flowcharts, ASCII tree | Working |
| `codeilus-search` | BM25 via SQLite FTS5 + RRF ranking | Basic |
| `codeilus-llm` | Claude CLI subprocess + stream-JSON parsing | Working, fragile |
| `codeilus-narrate` | Pre-generate LLM narratives at analysis time | Working (placeholder mode) |
| `codeilus-learn` | Curriculum, chapters, progress, quizzes | Basic structure |
| `codeilus-harvest` | GitHub trending scraper, shallow clone queue | Basic structure |
| `codeilus-export` | Static single-HTML renderer | Basic structure |
| `codeilus-api` | Axum HTTP + WS + rust-embed frontend | Working |
| `codeilus-mcp` | MCP server (rmcp stdio, 8 tools) | Basic — needs major expansion |
| `codeilus-app` | Binary: clap CLI, DB setup, server, shutdown | Working |

---

## What Works Well

1. **Parsing pipeline** — tree-sitter extraction reliably finds functions, classes, methods, structs across 6 languages
2. **Call graph** — 2,537 edges from a self-analysis, including `<module>` callers and macro invocations
3. **Community detection** — Louvain + aggressive merge produces reasonable clusters
4. **3D graph visualization** — WebGL force-directed graph with labeled nodes, community colors, click-to-inspect
5. **Source code viewer** — click symbol in file tree → see actual source with highlighted range
6. **Anti-pattern detection** — god classes, long methods, circular deps, security hotspots all working
7. **Single binary** — frontend embedded at compile time, zero runtime dependencies

## What Needs Work

1. **Edge diversity** — 99.9% of edges are CALLS. Only 2 IMPORTS edges, 1 EXTENDS. The dep_graph and heritage modules need more aggressive resolution.
2. **Community labels** — all labeled "src" (parent directory). Needs semantic labeling from symbol names/kinds.
3. **LLM narratives** — currently placeholders. Need real Claude integration or alternative.
4. **Learning path** — curriculum exists but chapters are skeletal. Quiz generation is basic.
5. **Harvest/Export** — basic structure only, not end-to-end tested.
6. **MCP server** — 8 tools defined but surface-level. Not deeply integrated with graph intelligence.
7. **Search** — BM25 indexing exists but limited query capability.
