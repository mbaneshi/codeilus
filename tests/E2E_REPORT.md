# E2E Validation Report

**Date**: 2026-03-15
**Target**: codeilus self-analysis (the codeilus repo analyzing itself)
**Database**: Fresh SQLite at /tmp/codeilus_e2e_test.db

---

## 1. Analyze Results (codeilus analyze .)

| Metric | Value | Target | Verdict |
|--------|-------|--------|---------|
| Files parsed | 145 | >10 | PASS |
| Symbols extracted | 829 | >50 | PASS |
| Edges built | 2,337 | 30-100+ | PASS (234x target minimum) |
| Communities | 34 | 3-10 | HIGH (see issues) |
| Entry points | 671 | 10-30 | HIGH (see issues) |
| Processes | 671 | — | Matches entry points |
| Patterns detected | 96 | >0 | PASS |
| SLOC | 15,754 | — | Reasonable |
| Avg complexity | 3.7 | — | Low (good) |
| Modularity | 0.284 | >0.3 | BORDERLINE |
| Diagrams | 1 (4,764 chars) | >0 | PASS |
| Narratives | ~41 via LLM | >0 | PASS (some timed out) |
| Chapters | 36 | >0 | PASS |

**Parse time**: ~500ms (fast, parallel via rayon)
**Total analyze time**: ~13 minutes (dominated by LLM narrative generation)

---

## 2. Server API Endpoint Results

| Endpoint | Status | Response | Verdict |
|----------|--------|----------|---------|
| GET /api/v1/health | 200 | `{"status":"ok"}` | PASS |
| GET /api/v1/files | 200 | 145 files | PASS |
| GET /api/v1/graph | 200 | 829 nodes, 2337 edges | PASS |
| GET /api/v1/communities | 200 | 34 communities | PASS |
| GET /api/v1/files/:id/source | 200 | Returns source with line numbers | PASS |
| GET /api/v1/chapters | 200 | 36 chapters | PASS |
| GET /api/v1/files/:id | 200 | Single file JSON | PASS |
| GET /api/v1/files/:id/symbols | 200 | Symbol list for file | PASS |

### Graph data quality
- All 829 nodes have `community_id` assigned
- Edge kinds: CALLS, IMPORTS, EXTENDS
- Sample node: `{"id":1,"name":"QuerySymbolsInput","kind":"Struct","file_id":1,"community_id":1}`

### Source endpoint
- Works when `repo_root` is set on AppState
- Returns JSON: `{"path","language","lines":[{"number","content"}],"total_lines"}`
- Line range filtering via `?start=N&end=M` works correctly

---

## 3. Integration Tests

Location: `crates/codeilus-app/tests/integration_test.rs`

| Test | Description | Status |
|------|-------------|--------|
| parse_fixture_produces_files_and_symbols | Parse codeilus-core → files + symbols | PASS |
| store_parsed_data_in_db | Store parsed data in SQLite | PASS |
| graph_builder_produces_communities | GraphBuilder → edges + communities | PASS |
| graph_api_returns_nodes_with_community_ids | Full round-trip: parse → store → graph API | PASS |
| files_api_returns_stored_files | Files API returns correct count | PASS |
| communities_api_returns_stored_communities | Communities API returns stored data | PASS |
| source_endpoint_with_repo_root | Source endpoint reads real files | PASS |

**Run**: `cargo test -p codeilus-app --test integration_test`
**Result**: 7/7 passed in 0.14s

---

## 4. Issues Found

### CRITICAL: Re-analyze fails with UNIQUE constraint
- Running `codeilus analyze .` twice on the same DB fails:
  `UNIQUE constraint failed: files.path`
- Need upsert (INSERT OR REPLACE) or delete-before-insert

### HIGH: Community count too high (34 instead of 5-15)
- 34 communities for 145 files is too fragmented
- Largest community has 321 members (39% of all symbols) — Louvain clumping
- Many tiny 1-2 member communities remain despite merging
- Labels are function names (e.g., `cluster_generate_all_narratives`), not module names

### HIGH: Entry points too permissive (671/829 = 81%)
- 81% of symbols flagged as entry points defeats the purpose
- Needs tighter heuristics: only main(), handlers, pub API functions

### MEDIUM: Narrative generation slow and unreliable
- 2 of 6 global narratives timed out (120s limit)
- Total narrative time: ~12 minutes for 41 narratives
- Some may contain error text instead of real prose if credits run low

### LOW: Step count mismatch in logs
- Log says "Step 1/5" then later "Step 3/8" — inconsistent total

---

## 5. What Works Well

- **Parsing**: Fast, accurate, all 145 Rust files parsed correctly
- **Symbol extraction**: 829 symbols with correct types and line ranges
- **Edge detection**: 2,337 edges across CALLS/IMPORTS/EXTENDS — massive improvement
- **API layer**: All endpoints return correct JSON, proper status codes
- **Source viewer**: Returns real source code with line numbers
- **Chapters**: 36 chapters generated with curriculum structure
- **Metrics**: Complexity, SLOC, fan-in/out, modularity all computed
- **Pattern detection**: 96 patterns (god class, long method, security, etc.)

---

## 6. Recommended Next Steps

1. **Fix re-analyze**: Add upsert or delete-all-before-insert to allow repeated analysis
2. **Community quality**: Tune Louvain resolution parameter; aim for 5-15 communities
3. **Community labels**: Derive from file paths / module names, not first symbol
4. **Entry point heuristics**: Only flag main(), HTTP handlers, public module roots
5. **Narrative timeout**: Increase to 180s or add retry with exponential backoff
6. **Frontend verification**: Open localhost:4174 in browser to check 3D graph rendering
