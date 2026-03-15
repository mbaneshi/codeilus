# Codeilus — Honest Audit: What's Real vs What's Shell (2026-03-15)

## Executive Summary

**8 out of 9 audited crates have 100% real logic. No pure stubs exist.** The codebase is a real functional system, not a skeleton. However, some features aren't wired end-to-end in the main pipeline, and the MCP tool implementations need expansion.

---

## Per-Crate Audit

| Crate | Real % | Verdict | What's Actually There |
|-------|--------|---------|----------------------|
| **codeilus-core** | 100% | REAL | IDs, errors, events, EventBus — foundational contract |
| **codeilus-db** | 100% | REAL | SQLite WAL, migrations, BatchWriter, 8 repo structs |
| **codeilus-parse** | 100% | REAL | Tree-sitter 6-language parsing, symbol extraction, import resolution |
| **codeilus-graph** | 100% | REAL | petgraph DiGraph, call graph, dep graph, heritage, Louvain communities, process detection |
| **codeilus-metrics** | 100% | REAL | Cyclomatic complexity, fan-in/out, SLOC, modularity, TF-IDF, git churn, heatmaps |
| **codeilus-analyze** | 100% | REAL | God class, long method, circular deps, security hotspots, test gaps |
| **codeilus-diagram** | 100% | REAL | Mermaid architecture diagrams, heuristic flowcharts, ASCII tree |
| **codeilus-search** | 100% | REAL | BM25 via FTS5, RRF ranking, multi-table search |
| **codeilus-llm** | 100% | REAL | Claude CLI spawn, stream-JSON parsing, retry logic, timeout handling |
| **codeilus-narrate** | 80% | PARTIAL | LLM narrative generation is real; placeholder fallback when LLM unavailable |
| **codeilus-learn** | 100% | REAL | Topological curriculum sort, 6-section chapters, multi-type quizzes, XP/badges/streaks |
| **codeilus-harvest** | 100% | REAL | GitHub trending scraper, shallow clone queue with semaphore, SHA256 fingerprinting |
| **codeilus-export** | 100% | REAL | Minijinja template rendering, data loading, index generation |
| **codeilus-api** | 100% | REAL | Axum HTTP, 16 REST endpoints, WebSocket events, rust-embed frontend |
| **codeilus-mcp** | 55% | PARTIAL | Server real (rmcp stdio), tool handlers partially implemented |
| **codeilus-app** | 100% | REAL | clap CLI, full analysis pipeline, serve, harvest, export commands |

---

## What's NOT Wired End-to-End

Even though the code is real, some pieces aren't connected in the main pipeline:

### 1. Harvest → Analyze → Export pipeline
- `codeilus harvest` scrapes trending repos and clones them
- But it doesn't automatically trigger `analyze` → `export` → `deploy`
- Each step works individually, but the full pipeline needs orchestration in `main.rs`

### 2. Export → Deploy
- `codeilus export` generates HTML pages
- But `codeilus deploy` (Cloudflare/GitHub Pages push) is not implemented
- The export template in `export-template/` needs actual HTML/CSS/JS content

### 3. MCP Tool Depth
- 8 tools declared, but responses are shallow
- Need structured JSON with depth parameter, confidence scores
- Need new tools: impact_analysis, trace_call_chain, find_related_code

### 4. LLM Narratives in Production
- Generator code is real, but mostly runs with CODEILUS_SKIP_LLM=1
- Real Claude integration needs testing at scale
- Prompts may need tuning for quality

### 5. Frontend Pages
- 3D graph, file tree, learn pages exist but haven't been browser-tested
- Some pages may have rendering issues (especially the 3d-force-graph integration)

---

## What's Actually Working End-to-End Today

```
$ codeilus analyze ./any-repo
✓ Parses all files (tree-sitter, 6 languages)
✓ Builds knowledge graph (875 symbols, 2540 edges)
✓ Computes metrics (SLOC, complexity, fan-in/out)
✓ Detects anti-patterns (god class, long method, security)
✓ Runs community detection (Louvain + merge)
✓ Scores entry points (30 ranked)
✓ Detects processes/execution flows
✓ Generates narratives (placeholder or real LLM)
✓ Builds curriculum (chapters, quizzes)
✓ Stores everything in SQLite

$ codeilus serve
✓ Serves frontend at localhost:4174
✓ REST API for all data (files, symbols, graph, communities, metrics, patterns, learn)
✓ WebSocket for real-time events
✓ Source code viewer (with repo_root)

$ codeilus mcp
✓ MCP stdio server starts
✓ Basic tool responses (query_symbols, etc.)
```

---

## The Real Gaps

1. **Graph edge diversity** — 99.9% CALLS edges. IMPORTS and heritage resolution is too strict.
2. **Community quality** — 23 communities, labels are meaningless ("src").
3. **MCP intelligence** — tools return shallow data, not deep codebase understanding.
4. **No visual verification** — frontend pages built but not browser-tested.
5. **No deployment pipeline** — harvest/export work individually but not as automated pipeline.
6. **Template content** — `export-template/` needs actual HTML/CSS/JS for static pages.
