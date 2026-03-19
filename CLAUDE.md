# Codeilus — CLAUDE.md

## Project Overview
Codeilus is a single Rust binary that analyzes any codebase and transforms it into a gamified, interactive learning experience in the browser. See `NORTH_STAR.md` for full vision and roadmap.

## Key Docs
- `NORTH_STAR.md` — Purpose, architecture, data flow, sprints, acceptance criteria
- `docs/AGENT_PROMPTS.md` — Copy-paste prompts for parallel Cursor agents (6 waves)
- `docs/OPTIMIZATIONS.md` — Performance optimization plan (12 items, 6 phases)
- `docs/SCHEMATIC_DESIGN.md` — Unified schematic explorer design
- `docs/adr/0002-unified-schematic-explorer.md` — ADR for schematic unification
- `CLAUDE.md` — This file (read by all agents)

## Git Workflow (MUST FOLLOW)

**Branching model:**
```
main (production — protected, receives PRs from dev only)
  └── dev (integration — receives PRs from feature branches)
       └── feat/xxx, fix/xxx, docs/xxx (short-lived feature branches)
```

**Rules for all agents:**
1. **NEVER commit directly to `main` or `dev`.** Always create a feature branch.
2. Create branches from `dev`: `git checkout -b feat/my-feature dev`
3. Push and open a PR targeting `dev` (not `main`).
4. PRs require CI to pass: `cargo build`, `cargo clippy` (zero warnings), `cargo test`.
5. Use squash merge when merging to `dev`. Feature branches auto-delete after merge.
6. Only `dev → main` PRs are used for releases.

**Branch naming conventions:**
- `feat/short-description` — new features
- `fix/short-description` — bug fixes
- `docs/short-description` — documentation only
- `refactor/short-description` — code restructuring
- `perf/short-description` — performance improvements

**Commit message format:**
```
type: concise description

Optional body with details.

Co-Authored-By: <agent name> <email>
```
Types: `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `ci`, `chore`

## Build & Test
```bash
cd /Users/bm/codeilus/codeilus
cargo build                    # build all crates
cargo test                     # run all tests
cargo clippy                   # must be zero warnings
cargo test -p codeilus-parse   # test single crate
cd frontend && pnpm build      # frontend build
```

## Architecture Rules
- `codeilus-core` has ZERO internal dependencies — it defines the contract
- `codeilus-db` depends only on `core`
- All other crates depend on `core` + `db`
- Never add cross-dependencies between sibling crates (e.g., parse must not depend on graph)
- All public types go through `core` if shared across crates
- IDs are i64 newtype wrappers (FileId, SymbolId, etc.) — never use raw i64 or UUID

## Code Style
- Zero clippy warnings, zero compiler warnings
- Use `thiserror` for error types, `tracing` for logging
- Async with `tokio`, CPU-parallel with `rayon`
- All DB operations through repository structs (FileRepo, SymbolRepo, etc.)
- Events flow through EventBus (tokio broadcast) — never direct state mutation
- Tests use in-memory SQLite (`DbPool::in_memory()`)

## Performance Patterns (established)
- **API pagination**: All list endpoints accept `?limit=` and `?offset=` (default 50, max 200)
- **Moka cache**: 500-entry, 10min TTL in `AppCache`. Cache reads before DB queries.
- **HTTP caching**: `Cache-Control: public, max-age=300` on all GET routes via middleware
- **No N+1 queries**: Batch-load with JOINs or `WHERE IN (...)`, group in Rust with HashMap
- **Frontend fetch cache**: `cachedGet()` in `api.ts` with 5min TTL for read-heavy endpoints
- **Vite**: Brotli compression, manual chunk splitting for three/shiki/3d-force-graph
- **Shiki**: Languages loaded per-file on demand, not all upfront

## Current State
Waves 1-4 complete. All 16 crates functional with 220+ tests passing, zero clippy warnings.
- Parse: Tree-sitter for 12 languages, incremental parsing
- Graph: Call graph, Louvain communities, 3-level zoom visualization
- Metrics: SLOC, complexity, fan-in/out, modularity
- Narrate: 8 narrative types via Claude Code CLI
- Learn: Curriculum, quizzes, XP/badges/streaks
- API: 50+ REST endpoints, SSE streaming Q&A, schematic lazy-load API
- Frontend: SvelteKit 5 with graph explorer, learning path, Ask AI, schematic views
- Infrastructure: r2d2 pool, moka cache (500 entries), pipeline checkpoints, structured logging
- Performance: Pagination on all list endpoints, N+1 fixes, client+server caching, brotli compression
Next: Unified schematic explorer (ADR-0002), Wave 5-6 polish, release pipeline.

## Parallel Agent Waves
- **Wave 1** (3 agents): codeilus-parse, codeilus-db repos, frontend skeleton
- **Wave 2** (2 agents): codeilus-graph, API routes
- **Wave 3** (3 agents): codeilus-metrics, codeilus-analyze, codeilus-diagram
- **Wave 4** (3 agents): codeilus-llm, codeilus-narrate, codeilus-learn
- **Wave 5** (2 agents): codeilus-harvest, codeilus-export
- **Wave 6** (1 agent): codeilus-mcp + pipeline wiring

Each agent owns specific crate directories. codeilus-core is READ-ONLY after Sprint 0.

## Reference Repos (for porting patterns)
- `../forge-project/` — Rust/Axum/SQLite architecture template
- `../GitNexus/` — parsing, graph, search
- `../emerge/` — metrics
- `../GitVizz/` — anti-patterns
- `../CodeVisualizer/` — flowchart IR
- `../gitdiagram/` — LLM diagram pipeline
- `../GitHubTree/` — file tree
- `../PocketFlow-Tutorial-Codebase-Knowledge/` — chapter writing prompts, pedagogical ordering
- `../deep-research/` — streaming research UX, agent event patterns
