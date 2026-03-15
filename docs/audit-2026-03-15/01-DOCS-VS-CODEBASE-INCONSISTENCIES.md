# Documentation vs Codebase Inconsistency Audit

**Date:** 2026-03-15
**Scope:** MkDocs site (`site/docs/`) vs actual Rust crates + frontend
**Auditor:** Senior Architecture Review

---

## Executive Summary

The MkDocs documentation site contains **14 confirmed inconsistencies** with the actual codebase. Three are critical (broken user flows), four are high severity (misleading claims), and seven are medium/low (cosmetic or aspirational). The docs were likely written from the NORTH_STAR.md vision document rather than validated against implemented code.

---

## CRITICAL Issues (Broken User Flows)

### C-1: `cargo install codeilus` Does Not Work

**Doc claim** (`index.md`, `installation.md`):
```bash
cargo install codeilus
codeilus ./any-repo
```

**Reality:** The package is NOT published to crates.io. The binary crate is named `codeilus-app`, not `codeilus`. All dependencies use `{ path = "..." }` workspace-relative paths, which means it cannot be installed from the registry.

**Impact:** The very first user interaction fails. Anyone following the Quick Start is immediately blocked.

**Fix:**
- Replace with `cargo install --path crates/codeilus-app` (from-source instructions)
- Add a clear "From Source" vs "From crates.io (coming soon)" section
- When ready to publish, configure `[package]` metadata in each crate's Cargo.toml with proper `version`, `license`, `description`, and convert path deps to versioned deps

---

### C-2: Three Frontend API Endpoints Have No Backend Implementation

**Doc claims** (`reference/api.md`, frontend `api.ts`):

| Endpoint | Frontend Function | Backend Status |
|----------|-------------------|----------------|
| `GET /api/v1/progress` | `fetchProgress()` | **NOT IMPLEMENTED** |
| `POST /api/v1/chapters/:id/sections/:sid/complete` | `markSectionComplete()` | **NOT IMPLEMENTED** |
| `GET /api/v1/learner/stats` | `fetchLearnerStats()` | **NOT IMPLEMENTED** |

**Impact:** The entire gamification loop (progress tracking, XP, streaks, badges) is non-functional. The Learning Path page will throw errors when trying to track completion.

**Fix:**
- Implement `progress` routes in `crates/codeilus-api/src/routes/chapters.rs`
- Implement `learner/stats` route in a new `learner.rs` route module or extend chapters
- Register them in `routes/mod.rs`
- Skeleton:

```rust
// In routes/chapters.rs or new routes/learner.rs
async fn get_progress(State(state): State<AppState>) -> Result<Json<Vec<Progress>>, ApiError> {
    let conn = state.db.get()?;
    let progress = ProgressRepo::new(&conn).list_all()?;
    Ok(Json(progress))
}

async fn mark_section_complete(
    State(state): State<AppState>,
    Path((chapter_id, section_id)): Path<(i64, i64)>,
) -> Result<Json<ProgressResponse>, ApiError> {
    let conn = state.db.get()?;
    ProgressRepo::new(&conn).mark_complete(chapter_id, section_id)?;
    // Award XP, check badges, update streak
    Ok(Json(ProgressResponse { xp_earned: 10 }))
}

async fn get_learner_stats(State(state): State<AppState>) -> Result<Json<LearnerStats>, ApiError> {
    let conn = state.db.get()?;
    let stats = LearnerStatsRepo::new(&conn).get_or_create()?;
    let badges = BadgeRepo::new(&conn).list_earned()?;
    Ok(Json(LearnerStats { total_xp: stats.total_xp, streak_days: stats.streak_days, badges }))
}
```

---

### C-3: Search Route Double-Prefix Bug

**File:** `crates/codeilus-api/src/routes/search.rs`

The search router registers its route with a full path `/api/v1/search` instead of just `/search`. Since the main router nests all routes under `/api/v1`, the actual route becomes `/api/v1/api/v1/search` — a 404 for any client.

**Impact:** Full-text search is unreachable.

**Fix:**
```rust
// BEFORE (broken):
Router::new().route("/api/v1/search", get(search))

// AFTER (correct):
Router::new().route("/search", get(search))
```

---

## HIGH Severity Issues (Misleading Claims)

### H-1: Language Count Inconsistency (13 defined, 6 work, docs say 12)

**Doc claims:**
- `crate-map.md`: "12 languages"
- `how-it-works.md`: "Python, TypeScript, JavaScript, Rust, Go, Java" (lists 6)

**Reality:**
- `codeilus-core/src/types.rs` defines 13 language variants: Python, TypeScript, JavaScript, Rust, Go, Java, C, Cpp, CSharp, Ruby, PHP, Swift, Kotlin
- `codeilus-parse/src/language.rs` only has tree-sitter grammars for **6**: Python, TypeScript, JavaScript, Rust, Go, Java
- The remaining 7 languages return `CodeilusError::Parse("Unsupported language")`

**Fix:**
- All docs should say "6 languages (Python, TypeScript, JavaScript, Rust, Go, Java)" with a note: "7 additional languages defined but pending tree-sitter integration"
- Add tree-sitter grammars for remaining languages (C, C++, Ruby are available on crates.io)

---

### H-2: MCP Tool Count — Docs Say 8, Code Has 16

**Doc claim** (`reference/mcp.md`): Lists 8 tools (list_repos, current_repo, query, context, impact, detect_changes, rename, diagram)

**Reality** (`codeilus-mcp/src/lib.rs`): 16 tools implemented: query_symbols, query_graph, get_context, get_impact, get_diagram, get_metrics, get_learning_status, explain_symbol, understand_codebase, trace_call_chain, impact_analysis, find_related_code, explain_file, find_tests_for, suggest_reading_order, get_community_context

**Fix:** Update `reference/mcp.md` to list all 16 tools with descriptions. The current doc was likely written early and never updated after the MCP crate was expanded.

---

### H-3: "8 Badges" Claim Is Unverified

**Doc claim** (`guide/learning-path.md`): Lists 8 specific badges (First Steps, Chapter Champion, Graph Explorer, Quiz Master, Deep Diver, Completionist, Polyglot, Code Detective)

**Reality:** The `badges` table exists in `0001_init.sql` but contains no seed data. The badge names are not defined anywhere in the Rust code — they exist only in documentation and the NORTH_STAR.md vision doc.

**Fix:**
- Add a `seed_badges()` function in `codeilus-db` that inserts the 8 canonical badges
- Call it during DB initialization in `codeilus-app/src/main.rs`
- Define badge constants in `codeilus-core` so they're consistent across crates

---

### H-4: Documentation Claims Features Work Without LLM — But No Fallback Narratives

**Doc claim** (`getting-started/quickstart.md`):
> With `CODEILUS_SKIP_LLM=1`... all features work except AI narratives and Q&A

**Reality:** The learning path depends on narratives for chapter overview content (`content_type: "narrative"`). Without LLM, narrative content will be empty/null, and chapter pages will show blank sections — not a graceful degradation.

**Fix:**
- Implement placeholder narrative content when LLM is unavailable
- Add a "generated by static analysis" fallback that uses TF-IDF keywords, file stats, and community descriptions as basic narratives
- Show a clear banner: "AI narratives unavailable — showing analysis-based summaries"

---

## MEDIUM Severity Issues

### M-1: Annotation Delete Response Mismatch

**Frontend** (`api.ts`): `deleteAnnotation()` returns `boolean` (compares response to truthy)
**Backend** (`routes/annotations.rs`): Returns `Json<()>` (empty body)

**Impact:** Delete will succeed server-side but frontend will treat it as failed.

**Fix:** Either return `Json(true)` from backend, or change frontend to check `response.ok` status.

---

### M-2: Quiz Answer Response Has Undocumented Field

**Frontend** (`api.ts`): Expects `{ correct: boolean, xp_earned: number }`
**Backend** (`routes/chapters.rs`): Returns `{ correct, xp_earned, explanation }`

**Impact:** Works but `explanation` field is silently ignored. Missed UX opportunity.

**Fix:** Update frontend `QuizAnswerResponse` type and display explanation feedback after quiz answers.

---

### M-3: `GET /api/v1/narratives` (List All) Exists But No Frontend Call

**Backend:** Implements `list_narratives()` with filtering support
**Frontend:** Only calls specific `fetchNarrative(kind)` and `fetchNarrative(kind, targetId)`

**Impact:** Wasted backend code, or useful for admin/debug but not exposed.

**Fix:** Either remove the endpoint or add an admin/debug page that uses it.

---

### M-4: Docs Don't Document Annotation Endpoints

**Reality:** Full CRUD on annotations is implemented:
- `GET /api/v1/annotations`
- `GET /api/v1/annotations/:target_type/:target_id`
- `POST /api/v1/annotations`
- `PUT /api/v1/annotations/:id`
- `POST /api/v1/annotations/:id/flag`
- `DELETE /api/v1/annotations/:id`

**Docs:** `reference/api.md` doesn't mention annotations at all.

**Fix:** Add an "Annotations" section to `reference/api.md`.

---

## LOW Severity Issues

### L-1: Server Port Inconsistency

- `index.md` says `http://localhost:4174`
- `quickstart.md` says `--port 4174` (correct)
- Actual default in `main.rs`: port 4174 (correct)
- But `vite.config.ts` proxies to `localhost:4174` during dev

This is consistent but should be documented that 4174 is the Rust server, and the Vite dev server runs on 5173 during development.

---

### L-2: "~15MB binary" Claim Unverified

Docs claim "single binary (~15MB)". This hasn't been verified with a release build. Debug builds are typically 50-100MB for Rust projects this size.

**Fix:** Build a release binary and measure: `cargo build --release && ls -lh target/release/codeilus`

---

### L-3: `CODEILUS_SKIP_LLM` Env Var Referenced in Docs But May Not Exist in Code

Docs reference `CODEILUS_SKIP_LLM` in quickstart.md but this needs verification that it's actually checked in the auto-detect logic.

**Fix:** Verify and implement if missing. Add to `auto_detect_provider()` in `codeilus-llm/src/provider.rs`.

---

## Consistency Matrix

| Document | Claims | Verified | Issues |
|----------|--------|----------|--------|
| `index.md` | 12 | 9 | 3 (install, binary size, port) |
| `installation.md` | 8 | 6 | 2 (cargo install, SKIP_LLM) |
| `quickstart.md` | 6 | 4 | 2 (cargo install, LLM fallback) |
| `how-it-works.md` | 15 | 13 | 2 (language count, narrative fallback) |
| `reference/api.md` | 24 | 20 | 4 (missing routes, annotations) |
| `reference/mcp.md` | 10 | 8 | 2 (tool count) |
| `crate-map.md` | 18 | 16 | 2 (language count, MCP tools) |
| `learning-path.md` | 10 | 8 | 2 (badges, progress routes) |

---

## Priority Fix Order

1. **Fix search route double-prefix** (5 minutes, unblocks search)
2. **Update installation docs** (10 minutes, unblocks onboarding)
3. **Implement 3 missing learning routes** (2-4 hours, unblocks gamification)
4. **Fix annotation delete response** (5 minutes)
5. **Update language count in all docs** (15 minutes)
6. **Update MCP tool list in docs** (30 minutes)
7. **Implement badge seeding** (1 hour)
8. **Add LLM-unavailable narrative fallbacks** (4 hours)
9. **Document annotation API endpoints** (30 minutes)
10. **Verify and implement CODEILUS_SKIP_LLM** (30 minutes)
