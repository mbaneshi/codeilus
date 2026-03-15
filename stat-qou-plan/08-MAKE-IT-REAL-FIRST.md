# Make It Real First — Then Expand

## Your Instinct Is Right

The code is surprisingly real (8/9 crates at 100%), but the **end-to-end experience** isn't there yet. Before adding MCP intelligence, SSE transport, or new graph edge types, we need to:

1. **Wire the last loose ends** so `codeilus analyze . && codeilus serve` delivers a complete experience
2. **Verify it visually** in a browser
3. **Fix the output quality** (communities, labels, edge diversity)

Then expand coherently.

---

## Phase 1: Make It Happen (2-3 days)

### Day 1: Fix Output Quality

| Task | File | What | Why |
|------|------|------|-----|
| Fix import resolution | `crates/codeilus-graph/src/dep_graph.rs` | Map Rust `use` paths to filesystem paths | Only 2 IMPORTS edges today |
| Fix community labels | `crates/codeilus-graph/src/builder.rs` | Use symbol names/kinds instead of parent dir | All labels say "src" |
| Trim curriculum | `crates/codeilus-learn/src/curriculum.rs` | Skip communities with <5 members | 232 chapters is noise |
| Lower community count | `crates/codeilus-graph/src/community.rs` | Tune merge thresholds or limit max communities | 23 is still high |

### Day 2: Visual Verification & Fixes

| Task | What | Why |
|------|------|-----|
| Start server | `codeilus serve` → open browser | Haven't actually seen it |
| Test 3D graph | Click nodes, filter communities, check labels | Major feature, untested |
| Test file tree | Click files → symbols → source viewer | Source viewer is new |
| Test learn page | Navigate chapters, check content quality | Learning is the differentiator |
| Fix any rendering bugs | Whatever breaks in browser | Ship quality |

### Day 3: Wire Remaining Pipeline

| Task | What | Why |
|------|------|-----|
| Harvest → analyze chain | `codeilus harvest` triggers analysis per repo | Currently manual steps |
| Export template | Real HTML/CSS/JS in `export-template/` | Currently minimal |
| Frontend rebuild | `cd frontend && pnpm build` → re-embed | Ensure latest code is embedded |

---

## Phase 2: MCP Intelligence (3-5 days)

Only after Phase 1 delivers a working product.

### Expand Tools
- Structured JSON responses with `depth` parameter
- New tools: `impact_analysis`, `trace_call_chain`, `find_related_code`, `understand_codebase`
- Test each tool with Claude Code

### Add SSE Transport
- Serve MCP over SSE alongside the web UI (same Axum server)
- This lets remote AI agents connect

### Auto-Config
- `codeilus setup` generates MCP config for Claude Code / Cursor
- Zero friction for users

---

## Phase 3: Graph Enrichment (3-5 days)

### New Edge Types
- USES_TYPE (from function signatures, variable types)
- CONTAINS (file → symbol hierarchy)
- TESTS (test functions → tested code)

### Smarter Resolution
- Rust: `use crate::X` → map to filesystem
- TypeScript: tsconfig paths, barrel exports
- Python: relative imports

---

## The Coherence Principle

Each change must:
1. **Build on what works** — don't rewrite, extend
2. **Be testable** — add a test for every new behavior
3. **Be visible** — if it doesn't show up in the UI or MCP output, it's not done
4. **Be consistent** — same patterns across all crates (repos for DB, structured JSON for API, petgraph for graph operations)

---

## Decision: What to Do Right Now

**Start with Day 1 of Phase 1.** Fix import resolution, community labels, and curriculum trimming. These are surgical changes in 3-4 files that immediately improve the quality of everything downstream (graph, communities, MCP responses, learning chapters, export pages).

Then open the browser and see what we actually have.
