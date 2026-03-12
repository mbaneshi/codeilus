# Codeilus — CLAUDE.md

## Project Overview
Codeilus is a single Rust binary that analyzes any codebase and transforms it into a gamified, interactive learning experience in the browser. See `NORTH_STAR.md` for full vision and roadmap.

## Key Docs
- `NORTH_STAR.md` — Purpose, architecture, data flow, sprints, acceptance criteria
- `docs/AGENT_PROMPTS.md` — Copy-paste prompts for parallel Cursor agents (6 waves)
- `CLAUDE.md` — This file (read by all agents)

## Build & Test
```bash
cd /Users/bm/codeilus/codeilus
cargo build                    # build all crates
cargo test                     # run all tests
cargo clippy                   # must be zero warnings
cargo test -p codeilus-parse   # test single crate
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

## Current State
Sprint 0 complete: 16 crates compile, 10 tests pass, zero clippy warnings.
Next: Wave 1 (parse + db repos + frontend skeleton) — see docs/AGENT_PROMPTS.md.

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
