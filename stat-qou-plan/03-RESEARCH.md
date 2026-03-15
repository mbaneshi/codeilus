# Codeilus — Research & Reference Analysis

## What We Built From (10 Reference Repos)

| Repo | What We Took | What We Should Still Take |
|------|-------------|--------------------------|
| **forge-project** | Rust/Axum/SQLite architecture, EventBus, BatchWriter, Claude CLI spawning | Stream-JSON parsing robustness, error recovery patterns |
| **GitNexus** | Symbol extraction, call tracing, community detection, import resolution | Better import resolution (tsconfig paths, Rust mod/use) |
| **emerge** | Metrics formulas (SLOC, fan-in/out, modularity, TF-IDF) | Git metrics (churn, contributors via git2-rs), heatmap scoring |
| **GitVizz** | Anti-pattern detection (god class, circular deps) | Graph search patterns, more sophisticated analysis |
| **CodeVisualizer** | FlowchartIR data structures | AST→IR conversion for per-function flowcharts |
| **gitdiagram** | 3-stage LLM pipeline concept | Auto-fix loop for Mermaid validation |
| **GitHubTree** | File tree sorting (folders-first) | ASCII tree styles (4 variants) |
| **PocketFlow-Tutorial-Codebase-Knowledge** | Pedagogical ordering concepts | Chapter writing prompts, "5-minute grasp" structure |
| **deep-research** | Streaming UX patterns | Agent event patterns, progress indicators |
| **code-understanding-tools** | General codebase analysis patterns | — |

## Key Research Findings

### 1. Graph Intelligence Is the Differentiator

Most code analysis tools stop at parsing. Codeilus's value is the **knowledge graph** — but it needs to be much richer:

- **Current**: 2,540 edges, 99.9% CALLS
- **Target**: 5,000+ edges with diverse types (CALLS, IMPORTS, EXTENDS, IMPLEMENTS, CONTAINS, USES_TYPE, READS, WRITES)

The graph should capture:
- Type usage (which functions use which types)
- Field access (which methods read/write which fields)
- Module containment (file → module → package hierarchy)
- Test coverage (which tests exercise which functions)

### 2. MCP Is the Distribution Channel

The MCP server is the most strategically important output. Every AI IDE (Claude Code, Cursor, Windsurf, Cline) speaks MCP. If codeilus can answer "what does this function do?" or "what would break if I change this?" better than the AI's built-in context, it becomes indispensable.

### 3. Narrative Quality Determines Adoption

Placeholder narratives are useless. The narratives need to be:
- **Accurate** (derived from real graph data, not hallucinated)
- **Layered** (30-second overview → 5-minute deep dive → full exploration)
- **Actionable** ("read these 3 files to understand 80%")

### 4. Static Export Is the Viral Loop

One developer generates a codeilus page for a repo → shares the link → other developers discover codeilus. The daily trending pipeline (`harvest → analyze → export → deploy`) automates this.

---

## Competitive Landscape

| Tool | What It Does | Codeilus Advantage |
|------|-------------|-------------------|
| **Sourcegraph** | Code search + navigation | Codeilus adds learning path, narratives, gamification |
| **CodeScene** | Behavioral analysis, hotspots | Codeilus is free, open-source, single binary |
| **Understand** | Static analysis + metrics | Codeilus adds AI narratives, 3D graph, learning |
| **Repomix** | Repo → single file for LLM | Codeilus provides structured analysis, not raw dump |
| **gitdiagram** | Repo → architecture diagram | Codeilus provides full interactive analysis, not just diagram |
| **Aider/Cursor** | AI coding assistants | Codeilus is complementary — provides codebase understanding as MCP tools |

---

## Technology Decisions & Rationale

| Decision | Choice | Why |
|----------|--------|-----|
| Language | Rust | Single binary, fast parsing, memory safe |
| DB | SQLite WAL | Zero config, embedded, proven at scale |
| Graph lib | petgraph | Best Rust graph library, efficient algorithms |
| Parser | tree-sitter | Industry standard, 100+ language grammars |
| Frontend | SvelteKit 5 | Fast, modern, compiles to static assets |
| 3D Graph | 3d-force-graph | WebGL, handles 1000+ nodes, good interaction |
| LLM | Claude CLI subprocess | Zero API keys needed, user's existing auth |
| MCP | rmcp | Official Rust MCP SDK |
| Embedding | rust-embed | Frontend compiled into binary at build time |
