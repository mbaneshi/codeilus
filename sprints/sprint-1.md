# Sprint 1: Parsing Engine

**Status:** Not Started
**Week:** 2 (target)

## Pre-Launch Checklist
- [ ] Define `ParsedFile`, `Symbol`, `Import`, `Call`, `Heritage` types in `codeilus-core/src/types.rs`
- [ ] Define `FileRepo`, `SymbolRepo` trait stubs in `codeilus-db/src/repos/mod.rs`
- [ ] Create 3 fixture repos in `tests/fixtures/` (Python, TypeScript, Rust)
- [ ] Commit contracts to main

## Agents

| Agent | Scope | Branch | Status | PR |
|-------|-------|--------|--------|-----|
| 1A: Parser | `crates/codeilus-parse/src/**` | sprint1/parse | Not Started | — |
| 1B: DB Repos | `crates/codeilus-db/src/repos/**` | sprint1/db-repos | Not Started | — |
| 1C: Frontend | `frontend/**` | sprint1/frontend | Not Started | — |
| 1D: Integration | `crates/codeilus-api/src/routes/**`, `crates/codeilus-app/src/main.rs` | sprint1/integration | Blocked on 1A+1B | — |

## Agent 1A: codeilus-parse

**Owns:** `crates/codeilus-parse/src/**`
**Port from:** `../GitNexus/src/core/ingestion/`
**Dependencies to add:** `tree-sitter`, `tree-sitter-python`, `tree-sitter-typescript`, `tree-sitter-javascript`, `tree-sitter-rust`, `tree-sitter-go`, `tree-sitter-java`, `ignore`, `rayon`

**Deliverables:**
- [ ] Language detection from file extensions
- [ ] Tree-sitter query strings for 6 languages (definitions, imports, calls, heritage)
- [ ] Filesystem walker respecting .gitignore
- [ ] Chunked parsing with 20MB byte budget + rayon
- [ ] Import resolution (language-specific)
- [ ] `pub fn parse_directory(path: &Path) -> Result<Vec<ParsedFile>>`
- [ ] Tests: parse each fixture repo, verify symbol counts

## Agent 1B: DB Repos

**Owns:** `crates/codeilus-db/src/repos/**`
**Port from:** `../forge-project/crates/forge-db/`

**Deliverables:**
- [ ] `FileRepo` — insert_batch, get_by_id, get_all, get_by_path
- [ ] `SymbolRepo` — insert_batch, get_by_id, get_by_file, get_all, search_by_name
- [ ] Tests: insert fixtures, query, verify counts

## Agent 1C: Frontend Skeleton

**Owns:** `frontend/**`
**Port from:** `../forge-project/frontend/`

**Deliverables:**
- [ ] SvelteKit 5 + adapter-static + TailwindCSS 4 init
- [ ] Root layout with sidebar nav (Learn, Explore, Ask, Settings)
- [ ] Welcome page with "Start Learning" CTA
- [ ] WebSocket store for real-time events
- [ ] `/explore/tree` page (file tree with sortable hierarchy)
- [ ] `pnpm build` produces static output

## Agent 1D: Integration (sequential, after 1A+1B merge)

**Owns:** API routes + CLI wiring
**Deliverables:**
- [ ] API: `GET /api/v1/files`, `GET /api/v1/files/:id/symbols`, `GET /api/v1/symbols`
- [ ] CLI: `codeilus analyze ./path` runs parser → stores via repos → emits events
- [ ] EventBus progress events during analysis
- [ ] Full `cargo test` passes
- [ ] E2E: analyze fixture repo → query API → verify response

## Acceptance Criteria
- [ ] Analyze a 10K-line Python repo in < 5 seconds
- [ ] All functions, classes, imports extracted correctly for 6 languages
- [ ] File tree page shows hierarchy with symbol counts
- [ ] Progress events stream to WebSocket during analysis
- [ ] Re-running analyze on same repo updates (not duplicates) data

## Blockers
(none yet)

## Decisions Made
(log decisions here as they arise)
