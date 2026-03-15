# Codeilus — Plan Forward

## Priority Stack (Ordered by Impact)

### P0: Fix What's Broken (1-2 days)

1. **Import edge resolution** — `dep_graph.rs` resolves by exact file path but Rust imports are `crate::module::Type`. Map Rust `use` paths to filesystem paths using crate name + module tree. This alone could add 200+ IMPORTS edges.

2. **Community labels** — Replace parent-dir labeling with: analyze symbol names/kinds within each community, pick the most descriptive module name. E.g., "graph_building" not "src".

3. **Curriculum inflation** — 232 chapters is noise. Only create chapters for communities with >5 members. The "misc" community (209 isolated symbols) gets one summary chapter, not 209.

4. **Visual verification** — Actually open localhost:4174, screenshot the 3D graph, file tree source viewer. Fix any rendering bugs.

---

### P1: Make MCP Server First-Class (3-5 days)

This is the highest-leverage work. Every improvement here multiplies across all AI clients.

#### New Tools to Add

| Tool | Input | Output | Why |
|------|-------|--------|-----|
| `understand_codebase` | (none) | Overview, architecture, key files, entry points | First thing any agent needs |
| `trace_call_chain` | symbol name | Ordered call sequence with files | Understand execution flow |
| `impact_analysis` | symbol name | What depends on this, blast radius score | Safe refactoring |
| `find_related_code` | symbol name + context | Symbols in same community, connected by edges | Better context gathering |
| `explain_file` | file path | Purpose, key symbols, connections, metrics | File-level understanding |
| `find_tests_for` | symbol name | Test files/functions that exercise this code | Test coverage awareness |
| `suggest_reading_order` | (none) | Top 5-10 files ranked by learning value | Onboarding |
| `get_community_context` | community id or name | All symbols, internal edges, external connections | Module understanding |

#### Transport Expansion

1. **stdio** (current) — works, keep it
2. **SSE** — add `/mcp/sse` endpoint to existing Axum server. This lets the codeilus server serve both the web UI AND MCP simultaneously.
3. **`codeilus setup`** — auto-generate MCP config for Claude Code, Cursor, Windsurf

#### Quality Bar

Every tool must return:
- Structured JSON (not prose)
- Confidence scores where applicable
- File paths with line numbers
- `depth` parameter (shallow/medium/deep)

---

### P2: Graph Intelligence (3-5 days)

#### More Edge Types

| Edge Type | Source | How to Extract |
|-----------|--------|---------------|
| USES_TYPE | Function params, return types, local vars | tree-sitter type annotations |
| CONTAINS | File → symbols, Module → files | Implicit from file_id |
| READS/WRITES | Field access patterns | tree-sitter field_expression |
| TESTS | Test functions → tested functions | Name matching + call graph |

#### Smarter Import Resolution

For Rust:
- Parse `use crate::foo::bar` → map `crate` to workspace crate name → resolve to `crates/{name}/src/foo/bar.rs`
- Handle `mod` declarations → map to `foo.rs` or `foo/mod.rs`
- Handle re-exports (`pub use`)

For TypeScript:
- Parse tsconfig.json `paths` for alias resolution
- Handle `index.ts` barrel exports
- Handle relative imports with `.ts`/`.tsx` extension resolution

#### Graph Queries

Add higher-level graph operations:
- **Shortest path** between two symbols
- **Common ancestors** (find shared dependencies)
- **Dependency depth** (how deep in the call chain)
- **Circular dependency detection** at any granularity (file, module, community)

---

### P3: Narrative & Learning Quality (5-7 days)

1. **Real LLM narratives** — Run Claude with actual graph data as context:
   - Feed community members + connections as structured prompt
   - Generate purpose summary, architecture description, extension guide
   - Cache aggressively — re-generate only when graph changes

2. **Interactive learning** — Each chapter should have:
   - Architecture diagram (auto-generated Mermaid from community subgraph)
   - Guided code walkthrough (source viewer with annotations)
   - Quiz with real questions ("Which function calls X?" from graph data)
   - Progress tracking with XP

3. **Reading order algorithm** — Rank files by:
   - Entry point score (high = read first)
   - Fan-in (high = important, read early)
   - Community centrality (hub nodes teach you the most)
   - Complexity (lower first for learning)

---

### P4: Harvest + Export Pipeline (3-5 days)

1. **GitHub trending scraper** — Parse trending page, queue repos
2. **Batch analysis** — Clone shallow, analyze, narrate, export
3. **Static HTML** — Single self-contained page per repo (<500KB)
4. **Daily deploy** — GitHub Actions cron → Cloudflare Pages

---

## What to Build This Week

| Day | Focus | Deliverable |
|-----|-------|-------------|
| 1 | P0 fixes | Import edges, community labels, curriculum trim, visual test |
| 2-3 | MCP tools | 8 new tools with structured output, SSE transport |
| 4 | MCP integration | `codeilus setup` command, test with Claude Code |
| 5 | Graph enrichment | USES_TYPE edges, Rust import resolution |

---

## Success Metrics

| Metric | Current | Target | How to Verify |
|--------|---------|--------|---------------|
| Edge count | 2,540 | 5,000+ | `SELECT COUNT(*) FROM edges` |
| Edge type diversity | 3 types (99.9% CALLS) | 5+ types, <80% any single | `GROUP BY kind` |
| Communities | 23 (many noise) | 5-15 (meaningful) | Visual inspection |
| Community labels | "src" | Descriptive names | Visual inspection |
| MCP tools | 8 (basic) | 15+ (structured) | `codeilus mcp --list-tools` |
| Entry points | 30 | 10-30 (quality) | Manual review |
| Chapters | 232 (noise) | 10-20 (meaningful) | `/learn` page |
| LLM narratives | Placeholder | Real content | Read in browser |
| Import edges | 2 | 200+ | `WHERE kind='IMPORTS'` |
