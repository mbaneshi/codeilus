# Fix Proposals — Actionable Implementation Plan

**Date:** 2026-03-15
**Priority:** Ordered by blast radius and implementation effort

---

## Fix 1: Search Route Double-Prefix (5 min, P0)

**File:** `crates/codeilus-api/src/routes/search.rs`

```rust
// BEFORE:
pub fn router() -> Router<AppState> {
    Router::new().route("/api/v1/search", get(search))
}

// AFTER:
pub fn router() -> Router<AppState> {
    Router::new().route("/search", get(search))
}
```

**Verification:** `cargo test -p codeilus-api` + manual curl: `curl http://localhost:4174/api/v1/search?q=test`

---

## Fix 2: Installation Docs Rewrite (10 min, P0)

**File:** `site/docs/getting-started/installation.md`

Replace the "Install" section:

```markdown
## Installation

### From Source (Current)

```bash
git clone https://github.com/codeilus/codeilus.git
cd codeilus
cargo install --path crates/codeilus-app
```

### From crates.io (Coming Soon)

```bash
# Not yet published — track https://github.com/codeilus/codeilus/issues/XX
cargo install codeilus
```
```

Also update `site/docs/index.md` hero section to match.

---

## Fix 3: Implement Missing Learning Endpoints (3 hrs, P0)

### 3a: Progress Route

**File:** `crates/codeilus-api/src/routes/chapters.rs` (extend existing)

```rust
use crate::state::AppState;

/// GET /api/v1/progress
async fn list_progress(
    State(state): State<AppState>,
) -> Result<Json<Vec<ProgressRow>>, ApiError> {
    let conn = state.db.get()?;
    let rows = conn.prepare(
        "SELECT id, chapter_id, section_id, completed, completed_at FROM progress"
    )?
    .query_map([], |row| {
        Ok(ProgressRow {
            id: row.get(0)?,
            chapter_id: row.get(1)?,
            section_id: row.get(2)?,
            completed: row.get(3)?,
            completed_at: row.get(4)?,
        })
    })?
    .collect::<Result<Vec<_>, _>>()?;
    Ok(Json(rows))
}

#[derive(Serialize)]
struct ProgressRow {
    id: i64,
    chapter_id: i64,
    section_id: i64,
    completed: bool,
    completed_at: Option<String>,
}
```

### 3b: Mark Section Complete

```rust
/// POST /api/v1/chapters/:chapter_id/sections/:section_id/complete
async fn mark_section_complete(
    State(state): State<AppState>,
    Path((chapter_id, section_id)): Path<(i64, i64)>,
) -> Result<Json<CompleteResponse>, ApiError> {
    let conn = state.db.get()?;

    // Upsert progress
    conn.execute(
        "INSERT INTO progress (chapter_id, section_id, completed, completed_at)
         VALUES (?1, ?2, 1, datetime('now'))
         ON CONFLICT(chapter_id, section_id) DO UPDATE SET completed = 1, completed_at = datetime('now')",
        params![chapter_id, section_id],
    )?;

    // Award XP
    conn.execute(
        "UPDATE learner_stats SET total_xp = total_xp + 10, last_active = date('now')",
        [],
    )?;

    // Check if chapter is fully complete for bonus XP
    let total_sections: i64 = conn.query_row(
        "SELECT COUNT(*) FROM chapter_sections WHERE chapter_id = ?1",
        params![chapter_id], |r| r.get(0)
    )?;
    let completed_sections: i64 = conn.query_row(
        "SELECT COUNT(*) FROM progress WHERE chapter_id = ?1 AND completed = 1",
        params![chapter_id], |r| r.get(0)
    )?;

    let mut xp = 10;
    if completed_sections >= total_sections {
        conn.execute("UPDATE learner_stats SET total_xp = total_xp + 50", [])?;
        xp += 50;
    }

    Ok(Json(CompleteResponse { xp_earned: xp }))
}
```

### 3c: Learner Stats

```rust
/// GET /api/v1/learner/stats
async fn get_learner_stats(
    State(state): State<AppState>,
) -> Result<Json<LearnerStatsResponse>, ApiError> {
    let conn = state.db.get()?;

    // Ensure stats row exists
    conn.execute(
        "INSERT OR IGNORE INTO learner_stats (id, total_xp, streak_days, last_active)
         VALUES (1, 0, 0, date('now'))",
        [],
    )?;

    let stats = conn.query_row(
        "SELECT total_xp, streak_days, last_active FROM learner_stats WHERE id = 1",
        [], |row| Ok(LearnerStatsResponse {
            total_xp: row.get(0)?,
            streak_days: row.get(1)?,
            last_active: row.get(2)?,
            badges: vec![], // filled below
        })
    )?;

    let badges = conn.prepare("SELECT name, description, icon, earned_at FROM badges")?
        .query_map([], |row| Ok(BadgeResponse {
            name: row.get(0)?,
            description: row.get(1)?,
            icon: row.get(2)?,
            earned_at: row.get(3)?,
        }))?
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Json(LearnerStatsResponse { badges, ..stats }))
}
```

### 3d: Route Registration

**File:** `crates/codeilus-api/src/routes/mod.rs`

Add to the chapter router:
```rust
.route("/progress", get(list_progress))
.route("/chapters/:chapter_id/sections/:section_id/complete", post(mark_section_complete))
.route("/learner/stats", get(get_learner_stats))
```

---

## Fix 4: Annotation Delete Response (5 min, P1)

**File:** `crates/codeilus-api/src/routes/annotations.rs`

```rust
// BEFORE:
async fn delete_annotation(...) -> Result<Json<()>, ApiError> {
    // ...delete logic...
    Ok(Json(()))
}

// AFTER:
async fn delete_annotation(...) -> Result<Json<DeleteResponse>, ApiError> {
    // ...delete logic...
    Ok(Json(DeleteResponse { deleted: true }))
}

#[derive(Serialize)]
struct DeleteResponse {
    deleted: bool,
}
```

---

## Fix 5: Language Count Documentation (15 min, P1)

**Files to update:**
- `site/docs/architecture/crate-map.md`: "12 languages" → "6 languages with tree-sitter (Python, TypeScript, JavaScript, Rust, Go, Java); 7 more defined but pending grammar integration"
- `site/docs/guide/how-it-works.md`: Add note about supported vs planned languages
- `site/docs/index.md`: Update any language claims

---

## Fix 6: MCP Tool Documentation (30 min, P1)

**File:** `site/docs/reference/mcp.md`

Replace the 8-tool list with the actual 16 tools from `codeilus-mcp/src/lib.rs`:

```markdown
## Available Tools (16)

### Query & Search
| Tool | Description |
|------|-------------|
| `query_symbols` | Search symbols by name, kind, or file |
| `query_graph` | Query graph edges with filters |
| `find_related_code` | Find code related to a symbol |
| `find_tests_for` | Find test files/functions for a given symbol |

### Context & Understanding
| Tool | Description |
|------|-------------|
| `get_context` | 360-degree view of a symbol (callers, callees, community, metrics) |
| `explain_symbol` | LLM-generated explanation of a specific symbol |
| `explain_file` | LLM-generated explanation of a file's role |
| `understand_codebase` | High-level codebase overview |
| `get_community_context` | Understand a community/module |

### Analysis
| Tool | Description |
|------|-------------|
| `get_metrics` | Code metrics for files or symbols |
| `get_impact` | Blast radius analysis with depth scoring |
| `impact_analysis` | Extended impact analysis with change propagation |
| `trace_call_chain` | Trace call paths between symbols |
| `detect_changes` | Map diffs to affected symbols and processes |

### Navigation & Learning
| Tool | Description |
|------|-------------|
| `suggest_reading_order` | Recommended file reading order |
| `get_learning_status` | Learning progress and curriculum overview |
| `get_diagram` | Generate architecture or flowchart diagrams |
```

---

## Fix 7: Badge Seeding (1 hr, P2)

### 7a: Define Badge Constants

**File:** `crates/codeilus-core/src/types.rs`

```rust
pub struct BadgeDefinition {
    pub name: &'static str,
    pub description: &'static str,
    pub icon: &'static str,
}

pub const CANONICAL_BADGES: &[BadgeDefinition] = &[
    BadgeDefinition { name: "First Steps", description: "Complete Chapter 0: The Big Picture", icon: "rocket" },
    BadgeDefinition { name: "Chapter Champion", description: "Complete any chapter", icon: "trophy" },
    BadgeDefinition { name: "Graph Explorer", description: "Visit 10 unique graph nodes", icon: "map" },
    BadgeDefinition { name: "Quiz Master", description: "Pass 5 quizzes", icon: "brain" },
    BadgeDefinition { name: "Deep Diver", description: "Read 20 symbol explanations", icon: "microscope" },
    BadgeDefinition { name: "Completionist", description: "Reach 100% learning progress", icon: "star" },
    BadgeDefinition { name: "Polyglot", description: "Explore code in 3+ languages", icon: "globe" },
    BadgeDefinition { name: "Code Detective", description: "Find 3 anti-patterns", icon: "magnifying-glass" },
];
```

### 7b: Seed Function

**File:** `crates/codeilus-db/src/lib.rs`

```rust
pub fn seed_badges(conn: &Connection) -> CodeilusResult<()> {
    let mut stmt = conn.prepare(
        "INSERT OR IGNORE INTO badges (name, description, icon) VALUES (?1, ?2, ?3)"
    )?;
    for badge in codeilus_core::CANONICAL_BADGES {
        stmt.execute(params![badge.name, badge.description, badge.icon])?;
    }
    Ok(())
}
```

### 7c: Call During Init

**File:** `crates/codeilus-app/src/main.rs` (after migrations)

```rust
codeilus_db::seed_badges(&db.get()?)?;
```

---

## Fix 8: LLM-Unavailable Narrative Fallbacks (4 hrs, P2)

**File:** `crates/codeilus-narrate/src/lib.rs`

```rust
pub fn generate_fallback_narrative(
    kind: NarrativeKind,
    graph: &KnowledgeGraph,
    parsed_files: &[ParsedFile],
) -> Narrative {
    let content = match kind {
        NarrativeKind::Overview => {
            let file_count = parsed_files.len();
            let symbol_count: usize = parsed_files.iter().map(|f| f.symbols.len()).sum();
            let languages: HashSet<_> = parsed_files.iter().map(|f| &f.language).collect();
            format!(
                "This repository contains {} files with {} symbols across {} languages ({}).\n\n\
                 *AI-generated narratives are unavailable. Install Claude Code for richer explanations.*",
                file_count, symbol_count, languages.len(),
                languages.iter().map(|l| l.to_string()).collect::<Vec<_>>().join(", ")
            )
        }
        NarrativeKind::Architecture => {
            let communities = &graph.communities;
            format!(
                "The codebase is organized into {} modules/communities.\n\n{}\n\n\
                 *Install Claude Code for AI-generated architecture narratives.*",
                communities.len(),
                communities.iter()
                    .map(|c| format!("- **{}**: {} members", c.name, c.members.len()))
                    .collect::<Vec<_>>().join("\n")
            )
        }
        // ... other kinds with similar static analysis fallbacks
        _ => "AI narrative not available. Install Claude Code for this content.".to_string(),
    };

    Narrative { kind, content, target_id: None, language: "en".to_string() }
}
```

---

## Fix 9: Document Annotation Endpoints (30 min, P2)

**File:** `site/docs/reference/api.md`

Add section:

```markdown
## Annotations

Interactive annotations on graph nodes and edges.

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/annotations` | List all annotations (filter: `?flagged=true`) |
| GET | `/annotations/:target_type/:target_id` | List annotations for specific node/edge |
| POST | `/annotations` | Create annotation |
| PUT | `/annotations/:id` | Update annotation content |
| POST | `/annotations/:id/flag` | Toggle flagged status |
| DELETE | `/annotations/:id` | Delete annotation |

### Create Annotation
```json
POST /api/v1/annotations
{
  "target_type": "node",
  "target_id": 42,
  "content": "This function handles authentication"
}
```
```

---

## Fix 10: Implement CODEILUS_SKIP_LLM (30 min, P2)

**File:** `crates/codeilus-llm/src/provider.rs` (in `auto_detect_provider`)

```rust
pub async fn auto_detect_provider() -> Arc<dyn LlmProvider> {
    // Check skip flag first
    if std::env::var("CODEILUS_SKIP_LLM").map(|v| v == "1" || v == "true").unwrap_or(false) {
        tracing::info!("LLM disabled via CODEILUS_SKIP_LLM");
        return Arc::new(NoOpProvider);
    }

    // ... existing auto-detection logic
}

struct NoOpProvider;

#[async_trait]
impl LlmProvider for NoOpProvider {
    async fn prompt(&self, _request: &LlmRequest) -> CodeilusResult<LlmResponse> {
        Ok(LlmResponse { text: String::new(), tokens_used: 0 })
    }
    async fn is_available(&self) -> bool { false }
    fn name(&self) -> &'static str { "none" }
}
```

---

## Fix 11: Add `progress` Table Unique Constraint

**File:** New migration `migrations/0005_progress_unique.sql`

The `progress` table allows duplicate (chapter_id, section_id) rows. The `mark_section_complete` handler uses `ON CONFLICT` which requires a unique constraint.

```sql
CREATE UNIQUE INDEX IF NOT EXISTS idx_progress_chapter_section
ON progress(chapter_id, section_id);
```

---

## Fix 12: Frontend Quiz Explanation Display

**File:** `frontend/src/routes/learn/[id]/+page.svelte`

After the quiz answer submission, display the `explanation` field:

```svelte
{#if quizResult}
  <div class="mt-4 p-4 rounded-lg {quizResult.correct ? 'bg-green-900/20' : 'bg-red-900/20'}">
    <p class="font-semibold">{quizResult.correct ? 'Correct!' : 'Incorrect'}</p>
    {#if quizResult.explanation}
      <p class="mt-2 text-sm text-gray-300">{quizResult.explanation}</p>
    {/if}
    <p class="mt-1 text-xs text-gray-400">+{quizResult.xp_earned} XP</p>
  </div>
{/if}
```

---

## Implementation Timeline

| Week | Fixes | Effort |
|------|-------|--------|
| Day 1 | Fix 1, 2, 4, 5 | 1 hour |
| Day 2 | Fix 3 (all learning routes) | 3 hours |
| Day 3 | Fix 6, 9, 10 | 2 hours |
| Day 4 | Fix 7, 11, 12 | 2 hours |
| Day 5 | Fix 8 (narrative fallbacks) | 4 hours |
| **Total** | **12 fixes** | **~12 hours** |
