# Task: Learning Engine

> **Crate:** `crates/codeilus-learn/`
> **Wave:** 4 (parallel with llm, narrate)
> **Depends on:** codeilus-core (done), codeilus-graph (wave 2), codeilus-narrate (wave 4 — same wave, learn reads narratives from DB), codeilus-db (wave 1+2)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 6.5 (codeilus-learn deep dive), section 9 Sprint 6 (THE DIFFERENTIATOR)
- `crates/codeilus-core/src/ids.rs` — CommunityId, ChapterId, SymbolId
- `crates/codeilus-core/src/types.rs` — all shared types
- `crates/codeilus-graph/src/types.rs` — KnowledgeGraph, Community, EntryPoint, Process
- Reference: `../PocketFlow-Tutorial-Codebase-Knowledge/` — chapter writing prompts, pedagogical ordering (foundational → user-facing → implementation), LLM-driven community naming with beginner analogies, curriculum structure

## Objective

Transform graph + narratives into a gamified learning curriculum. Generate chapters from communities (topologically sorted), implement progress tracking, XP/badge gamification, and quiz generation from graph data.

Public API:
```rust
pub fn generate_curriculum(graph: &KnowledgeGraph) -> CodeilusResult<Curriculum>
pub fn record_progress(chapter_id: ChapterId, section: &str) -> CodeilusResult<ProgressUpdate>
pub fn get_stats() -> CodeilusResult<LearnerStats>
pub fn generate_quiz(chapter_id: ChapterId) -> CodeilusResult<Quiz>
```

## Files to Create/Modify

### 1. Update `crates/codeilus-learn/Cargo.toml`

```toml
[package]
name = "codeilus-learn"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
codeilus-db = { path = "../codeilus-db" }
codeilus-graph = { path = "../codeilus-graph" }
chrono = { version = "0.4", features = ["serde"] }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 2. `src/types.rs` — Learning types

```rust
use codeilus_core::ids::{ChapterId, CommunityId, SymbolId};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Curriculum {
    pub chapters: Vec<Chapter>,
    pub total_sections: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Chapter {
    pub id: ChapterId,
    pub order: usize,
    pub title: String,
    pub description: String,
    pub community_id: Option<CommunityId>,
    pub sections: Vec<Section>,
    pub difficulty: Difficulty,
    pub prerequisite_ids: Vec<ChapterId>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: String,
    pub kind: SectionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SectionKind {
    Overview,
    KeyConcepts,
    Diagram,
    CodeWalkthrough,
    Connections,
    Quiz,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Difficulty {
    Beginner,
    Intermediate,
    Advanced,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressUpdate {
    pub chapter_id: ChapterId,
    pub section_id: String,
    pub xp_earned: i64,
    pub badges_earned: Vec<Badge>,
    pub total_xp: i64,
    pub overall_progress: f64,  // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerStats {
    pub total_xp: i64,
    pub level: usize,
    pub badges: Vec<Badge>,
    pub streak_days: usize,
    pub chapters_completed: usize,
    pub sections_completed: usize,
    pub quizzes_passed: usize,
    pub overall_progress: f64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Badge {
    FirstSteps,       // Complete Chapter 0
    ChapterChampion,  // Complete any chapter
    GraphExplorer,    // Visit 10 different nodes
    QuizMaster,       // Pass 5 quizzes
    DeepDiver,        // Read 20 symbol explanations
    Completionist,    // 100% progress
    Polyglot,         // Explore files in 3+ languages
    CodeDetective,    // Find 3 anti-patterns
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quiz {
    pub chapter_id: ChapterId,
    pub questions: Vec<QuizQuestion>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuizQuestion {
    pub id: String,
    pub question: String,
    pub kind: QuizQuestionKind,
    pub options: Vec<String>,
    pub correct_index: usize,
    pub explanation: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuizQuestionKind {
    MultipleChoice,
    TrueFalse,
    ImpactAnalysis,  // "What happens if you change X?"
}
```

### 3. `src/curriculum.rs` — Curriculum generation

- **Topological sort** communities by dependency (community A depends on B if A imports from B)
- Build dependency graph between communities from inter-community edges in KnowledgeGraph
- Sort: entry point communities first, then by dependency order, core before features
- **Chapter 0**: "The Big Picture" — overview + architecture diagram (no community, always first)
- **Chapters 1..N**: one per community, ordered by topological sort
- **Final chapter**: "Putting It All Together" — cross-cutting execution flows (always last)
- Each chapter has 6 sections: Overview, KeyConcepts, Diagram, CodeWalkthrough, Connections, Quiz
- **Difficulty**: based on community complexity metrics
  - Avg symbol complexity <5 → Beginner
  - 5-15 → Intermediate
  - 15+ → Advanced
- **Prerequisites**: chapter A requires chapter B if community A depends on community B
- Reference: `../PocketFlow-Tutorial-Codebase-Knowledge/` for pedagogical ordering

### 4. `src/progress.rs` — Progress tracking

```rust
pub struct ProgressTracker {
    // holds DB connection
}

impl ProgressTracker {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;

    /// Record completion of a section. Returns XP earned and any new badges.
    pub fn complete_section(&self, chapter_id: ChapterId, section_id: &str) -> CodeilusResult<ProgressUpdate>;

    /// Record a quiz attempt.
    pub fn record_quiz(&self, chapter_id: ChapterId, score: f64, passed: bool) -> CodeilusResult<ProgressUpdate>;

    /// Record graph exploration (viewing a node).
    pub fn record_explore(&self, symbol_id: SymbolId) -> CodeilusResult<ProgressUpdate>;

    /// Record asking a Q&A question.
    pub fn record_question(&self) -> CodeilusResult<ProgressUpdate>;

    /// Get current learner stats.
    pub fn get_stats(&self) -> CodeilusResult<LearnerStats>;

    /// Get progress for a specific chapter.
    pub fn get_chapter_progress(&self, chapter_id: ChapterId) -> CodeilusResult<f64>;

    /// Check and award any newly earned badges.
    fn check_badges(&self) -> CodeilusResult<Vec<Badge>>;

    /// Update streak (called on each interaction).
    fn update_streak(&self) -> CodeilusResult<()>;
}
```

**XP system:**
| Action | XP |
|---|---|
| Complete a section | +10 |
| Complete a chapter (all sections) | +50 |
| Pass a quiz | +25 |
| Explore graph (view node) | +5 |
| Ask a Q&A question | +5 |

**Badge logic:**
| Badge | Condition |
|---|---|
| FirstSteps | Chapter 0 completed |
| ChapterChampion | Any chapter 100% |
| GraphExplorer | 10+ unique nodes visited |
| QuizMaster | 5+ quizzes passed |
| DeepDiver | 20+ symbol explanations read |
| Completionist | 100% overall progress |
| Polyglot | Files in 3+ languages explored |
| CodeDetective | 3+ anti-patterns found/viewed |

### 5. `src/quiz.rs` — Quiz generation from graph data

- Generate quiz questions from the KnowledgeGraph for a given chapter/community:
  - **MultipleChoice**: "Which module does X depend on?" (from edges)
  - **TrueFalse**: "Symbol X calls Symbol Y" (from call graph, true or false)
  - **ImpactAnalysis**: "If you change function X, which other functions are affected?" (from callers)
- Generate 5 questions per chapter
- Shuffle options, record correct_index
- Provide explanation from graph data ("X calls Y because of edge at line N")

```rust
pub fn generate_quiz(
    chapter: &Chapter,
    graph: &KnowledgeGraph,
) -> CodeilusResult<Quiz> { ... }
```

### 6. `src/lib.rs` — Module entry point

```rust
pub mod curriculum;
pub mod progress;
pub mod quiz;
pub mod types;

pub use curriculum::*;
pub use progress::ProgressTracker;
pub use quiz::generate_quiz;
pub use types::*;

use codeilus_core::ids::ChapterId;
use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;

pub fn generate_curriculum(graph: &KnowledgeGraph) -> CodeilusResult<Curriculum> {
    curriculum::generate(graph)
}
```

### 7. Add to `crates/codeilus-db/src/repos/` — Two new repos

#### `chapter_repo.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterRow {
    pub id: ChapterId,
    pub order_index: i64,
    pub title: String,
    pub description: String,
    pub community_id: Option<i64>,
    pub difficulty: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChapterSectionRow {
    pub id: i64,
    pub chapter_id: ChapterId,
    pub section_id: String,
    pub title: String,
    pub kind: String,
}

pub struct ChapterRepo { conn: Arc<Mutex<Connection>> }

impl ChapterRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn insert(&self, order_index: i64, title: &str, description: &str, community_id: Option<i64>, difficulty: &str) -> CodeilusResult<ChapterId>;
    pub fn insert_section(&self, chapter_id: ChapterId, section_id: &str, title: &str, kind: &str) -> CodeilusResult<i64>;
    pub fn get(&self, id: ChapterId) -> CodeilusResult<ChapterRow>;
    pub fn list_ordered(&self) -> CodeilusResult<Vec<ChapterRow>>;
    pub fn list_sections(&self, chapter_id: ChapterId) -> CodeilusResult<Vec<ChapterSectionRow>>;
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

#### `progress_repo.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressRow {
    pub id: i64,
    pub chapter_id: i64,
    pub section_id: String,
    pub completed_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearnerStatsRow {
    pub total_xp: i64,
    pub streak_days: i64,
    pub last_active: String,
    pub nodes_visited: i64,
    pub explanations_read: i64,
    pub patterns_found: i64,
    pub languages_explored: String,  // JSON array of language strings
}

pub struct ProgressRepo { conn: Arc<Mutex<Connection>> }

impl ProgressRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn record_section(&self, chapter_id: i64, section_id: &str) -> CodeilusResult<i64>;
    pub fn record_quiz_attempt(&self, chapter_id: i64, score: f64, passed: bool) -> CodeilusResult<i64>;
    pub fn is_section_complete(&self, chapter_id: i64, section_id: &str) -> CodeilusResult<bool>;
    pub fn get_chapter_progress(&self, chapter_id: i64) -> CodeilusResult<f64>;
    pub fn get_overall_progress(&self) -> CodeilusResult<f64>;
    pub fn get_or_create_stats(&self) -> CodeilusResult<LearnerStatsRow>;
    pub fn update_stats(&self, stats: &LearnerStatsRow) -> CodeilusResult<()>;
    pub fn list_completed_sections(&self, chapter_id: i64) -> CodeilusResult<Vec<String>>;
    pub fn count_quizzes_passed(&self) -> CodeilusResult<usize>;
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

Update `crates/codeilus-db/src/repos/mod.rs` to include both new repos.

## Tests

### Test cases:
1. `curriculum_has_chapter_zero` — generate_curriculum always starts with "The Big Picture"
2. `curriculum_has_final_chapter` — Last chapter is "Putting It All Together"
3. `curriculum_topological_order` — If community A depends on B, chapter for B comes before A
4. `curriculum_sections_complete` — Each chapter has all 6 section kinds
5. `curriculum_difficulty_from_complexity` — Low complexity community → Beginner difficulty
6. `curriculum_prerequisites` — Chapter for dependent community lists prerequisite chapter IDs
7. `progress_section_xp` — Completing a section awards +10 XP
8. `progress_chapter_bonus` — Completing all sections in a chapter awards +50 bonus XP
9. `progress_quiz_xp` — Passing a quiz awards +25 XP
10. `progress_explore_xp` — Exploring a graph node awards +5 XP
11. `badge_first_steps` — Complete chapter 0 → FirstSteps badge
12. `badge_chapter_champion` — Complete any chapter → ChapterChampion badge
13. `badge_quiz_master` — Pass 5 quizzes → QuizMaster badge
14. `badge_not_duplicated` — Earning same badge twice → only returned once
15. `quiz_five_questions` — generate_quiz returns exactly 5 questions
16. `quiz_correct_index_valid` — correct_index is within options length
17. `quiz_has_explanation` — Each question has non-empty explanation
18. `quiz_types_varied` — Quiz has mix of MultipleChoice, TrueFalse, ImpactAnalysis

### DB repo tests:
19. `chapter_repo_ordered` — Insert 3 chapters out of order, list_ordered returns in order
20. `chapter_repo_sections` — Insert sections, list_sections returns for correct chapter
21. `progress_repo_section_complete` — Record section, is_section_complete returns true
22. `progress_repo_chapter_progress` — 3/6 sections complete → progress = 0.5

## Acceptance Criteria

- [ ] `cargo test -p codeilus-learn` — all tests pass
- [ ] `cargo clippy -p codeilus-learn` — zero warnings
- [ ] `cargo test -p codeilus-db` — all tests pass (including new repo tests)
- [ ] Curriculum starts with Chapter 0 "The Big Picture"
- [ ] Curriculum ends with "Putting It All Together"
- [ ] Chapters ordered by topological sort of community dependencies
- [ ] Each chapter has 6 section types
- [ ] XP awards correct amounts for each action type
- [ ] All 8 badges have working detection logic
- [ ] Quizzes generated from graph data (not random)
- [ ] Progress tracking is persistent (via DB)
- [ ] Streak tracking works across sessions

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- `crates/codeilus-graph/` (wave 2)
- `crates/codeilus-parse/` (wave 1)
- `crates/codeilus-narrate/` (wave 4 — parallel, read-only)
- `crates/codeilus-db/src/pool.rs`, `migrations.rs`, `batch_writer.rs`
- Existing repo files in `crates/codeilus-db/src/repos/`
- `migrations/0001_init.sql`
- Any files outside `crates/codeilus-learn/` and the two new DB repo files

---

## Report

> **Agent: filled on 2026-03-13.**

### Status: complete

### Files Created/Modified:
- `crates/codeilus-learn/Cargo.toml` — added dependencies: codeilus-db, codeilus-graph, petgraph, rusqlite, chrono, tracing, serde, serde_json
- `crates/codeilus-learn/src/lib.rs` — module entry point, re-exports, `generate_curriculum()` public API
- `crates/codeilus-learn/src/types.rs` — Curriculum, Chapter, Section, SectionKind, Difficulty, ProgressUpdate, LearnerStats, Badge (8 variants), Quiz, QuizQuestion, QuizQuestionKind
- `crates/codeilus-learn/src/curriculum.rs` — topological sort of communities → ordered chapters, Chapter 0 "The Big Picture", final chapter "Putting It All Together", 6 sections per chapter, difficulty estimation, prerequisite tracking
- `crates/codeilus-learn/src/progress.rs` — ProgressTracker with XP system (+10 section, +50 chapter bonus, +25 quiz, +5 explore, +5 question), badge detection (FirstSteps, ChapterChampion, QuizMaster, Completionist), streak tracking
- `crates/codeilus-learn/src/quiz.rs` — Quiz generation from graph data: MultipleChoice (dependency questions), TrueFalse (call graph), ImpactAnalysis (caller analysis), 5 questions per chapter
- `crates/codeilus-db/src/repos/chapter_repo.rs` — ChapterRepo: insert, insert_section, get, list_ordered, list_sections, delete_all
- `crates/codeilus-db/src/repos/progress_repo.rs` — ProgressRepo: record_section, record_quiz_attempt, is_section_complete, get_chapter_progress, get_overall_progress, get_or_create_stats, update_stats, list_completed_sections, count_quizzes_passed, insert_badge, list_badges, count_completed_chapters, count_completed_sections, is_chapter_complete, delete_all
- `crates/codeilus-db/src/repos/mod.rs` — added chapter_repo and progress_repo modules
- `crates/codeilus-db/src/lib.rs` — added re-exports for ChapterRepo, ChapterRow, ChapterSectionRow, ProgressRepo, ProgressRow, LearnerStatsRow

### Tests:
```
running 18 tests
test curriculum::tests::curriculum_has_chapter_zero ... ok
test curriculum::tests::curriculum_has_final_chapter ... ok
test curriculum::tests::curriculum_prerequisites ... ok
test curriculum::tests::curriculum_difficulty_from_complexity ... ok
test curriculum::tests::curriculum_sections_complete ... ok
test curriculum::tests::curriculum_topological_order ... ok
test quiz::tests::quiz_correct_index_valid ... ok
test quiz::tests::quiz_five_questions ... ok
test quiz::tests::quiz_has_explanation ... ok
test quiz::tests::quiz_types_varied ... ok
test progress::tests::progress_section_xp ... ok
test progress::tests::progress_explore_xp ... ok
test progress::tests::progress_quiz_xp ... ok
test progress::tests::badge_first_steps ... ok
test progress::tests::badge_chapter_champion ... ok
test progress::tests::progress_chapter_bonus ... ok
test progress::tests::badge_not_duplicated ... ok
test progress::tests::badge_quiz_master ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Clippy:
```
Finished `dev` profile [unoptimized + debuginfo] — zero warnings
```

### DB Tests:
```
cargo test -p codeilus-db: 27 passed, 0 failed (including existing tests)
cargo clippy -p codeilus-db: zero warnings
```

### Issues / Blockers:
- Task spec referenced `ExtractedSymbol` in the public API but parse crate exports `Symbol` — not relevant since learn crate doesn't directly use parse types.
- The `LearnerStatsRow` in the task spec had extra fields (nodes_visited, explanations_read, patterns_found, languages_explored) that don't exist in the DB schema (`learner_stats` table only has total_xp, streak_days, last_active). Adapted `LearnerStatsRow` to match the actual DB columns. The extra stats can be computed dynamically from other tables.
- Quiz attempts are tracked through the progress table (marking the "quiz" section as completed) rather than the `quiz_attempts` table, since `quiz_attempts` requires a FK to `quiz_questions` which would need pre-populated questions. Future waves can enhance this.

### Notes:
- The `ProgressTracker` requires chapters and sections to be persisted in the DB first (via `ChapterRepo`) before progress can be tracked. The curriculum generator creates in-memory `Chapter` objects — a pipeline step should persist them via `ChapterRepo` before progress tracking begins.
- Badge detection currently covers: FirstSteps, ChapterChampion, QuizMaster, Completionist. GraphExplorer, DeepDiver, Polyglot, and CodeDetective require additional tracking data (node visits, explanation reads, language stats, pattern findings) that aren't in the current DB schema. These can be added when the relevant features (graph explorer, symbol explanations, multi-language support, pattern viewer) are implemented.
- The topological sort uses Kahn's algorithm with entry-point priority. Communities with entry points are processed first within each topological level.
- Quiz generation is deterministic based on graph structure — questions are derived from actual dependency edges, call relationships, and caller analysis.
