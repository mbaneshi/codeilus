# Task: Narrative Generator

> **Crate:** `crates/codeilus-narrate/`
> **Wave:** 4 (parallel with llm, learn)
> **Depends on:** codeilus-core (done), codeilus-llm (wave 4 — same wave, but narrate calls llm), codeilus-graph (wave 2), codeilus-db (wave 1+2)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 6.4 (codeilus-narrate deep dive), section 9 Sprint 5
- `crates/codeilus-core/src/types.rs` — NarrativeKind enum (Overview, Architecture, ReadingOrder, ExtensionGuide, ContributionGuide, WhyTrending, ModuleSummary, SymbolExplanation)
- `crates/codeilus-core/src/ids.rs` — CommunityId, SymbolId
- `crates/codeilus-llm/src/types.rs` — LlmRequest, LlmResponse
- `crates/codeilus-llm/src/context.rs` — build_context function
- `crates/codeilus-graph/src/types.rs` — KnowledgeGraph, Community, EntryPoint
- Reference: `../PocketFlow-Tutorial-Codebase-Knowledge/` — chapter-writing prompts, prompt templates, multi-language prompt strategy. Read the flow files and prompt templates carefully.

## Objective

Pre-generate all 8 narrative types at analysis time using codeilus-llm. Store results in the `narratives` DB table. Provide instant access to pre-generated content (no LLM latency at serve time). Gracefully degrade with placeholder text when LLM is unavailable.

Public API:
```rust
pub async fn generate_all_narratives(
    graph: &KnowledgeGraph,
    parsed_files: &[ParsedFile],
    repo_path: &Path,
) -> CodeilusResult<Vec<Narrative>>
```

## Files to Create/Modify

### 1. Update `crates/codeilus-narrate/Cargo.toml`

```toml
[package]
name = "codeilus-narrate"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
codeilus-db = { path = "../codeilus-db" }
codeilus-llm = { path = "../codeilus-llm" }
codeilus-graph = { path = "../codeilus-graph" }
codeilus-parse = { path = "../codeilus-parse" }
tokio = { workspace = true }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 2. `src/types.rs` — Narrative types

```rust
use codeilus_core::types::NarrativeKind;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Narrative {
    pub kind: NarrativeKind,
    pub target_id: Option<i64>,     // community_id or symbol_id, depending on kind
    pub title: String,
    pub content: String,
    pub is_placeholder: bool,        // true if LLM was unavailable
}
```

### 3. `src/prompts.rs` — Prompt templates

Define prompt templates for each narrative kind. Adapt from PocketFlow patterns.

```rust
pub struct PromptTemplate {
    pub system: &'static str,
    pub user_template: &'static str,  // contains {context} placeholder
}

pub fn get_prompt(kind: NarrativeKind) -> PromptTemplate { ... }
```

**8 prompt templates:**

1. **Overview**: "You are a senior developer explaining a codebase to a newcomer. Given the following codebase context, write a 2-3 paragraph overview explaining what this project does, who it's for, and why it matters. Be concise and engaging. Context: {context}"

2. **Architecture**: "You are a software architect explaining system design. Given the following community/module structure and their connections, explain how this codebase is structured in 3-5 paragraphs. Mention the key modules, their responsibilities, and how they interact. Context: {context}"

3. **ReadingOrder**: "You are a mentor guiding a new developer. Given the following codebase analysis (entry points, fan-in scores, community centrality), recommend the 3-5 most important files to read first to understand 80% of the codebase. For each file, explain WHY it's important in 1-2 sentences. Context: {context}"

4. **ExtensionGuide**: "You are a developer advocate writing an extension guide. Given the following high fan-in interfaces, plugin patterns, and configuration points, write a step-by-step guide for adding new features to this codebase. Context: {context}"

5. **ContributionGuide**: "You are writing a contribution guide for newcomers. Given the following entry points, code patterns, and test coverage information, write a guide for first-time contributors. Include how to find good first issues, understand the codebase, and submit changes. Context: {context}"

6. **WhyTrending**: "You are a tech journalist explaining why a project is gaining attention. Given the following project description and ecosystem context, write 1-2 paragraphs about why developers are excited about this project. Context: {context}"

7. **ModuleSummary**: "You are documenting a module/community. Given the following symbols, edges, and metrics for this module, write a 1-2 paragraph summary of what this module does, its key types, and how it connects to the rest of the codebase. Use beginner-friendly analogies. Context: {context}"

8. **SymbolExplanation**: "You are explaining a function/class to a junior developer. Given the following symbol signature, its callers, callees, and containing file context, explain what this symbol does in 2-3 sentences. Include a beginner-friendly analogy if appropriate. Context: {context}"

Reference: `../PocketFlow-Tutorial-Codebase-Knowledge/` for battle-tested prompt patterns (analogies, <10 line snippets, cross-references, Mermaid diagrams).

### 4. `src/placeholders.rs` — Fallback content when LLM unavailable

```rust
pub fn placeholder_for(kind: NarrativeKind) -> String { ... }
```

Return sensible placeholder text for each kind:
- Overview: "This project contains {n} files across {languages}. Run with Claude Code available for a detailed overview."
- Architecture: "This codebase has {n} communities/modules. Claude Code is needed to generate architectural descriptions."
- ReadingOrder: List entry point files by score (no LLM needed for this)
- ModuleSummary: List member symbols and edge counts (data-driven, no LLM needed)
- Others: "Claude Code is required to generate this content. Install and ensure `claude` is in your PATH."

### 5. `src/generator.rs` — Narrative generation orchestrator

```rust
use crate::types::Narrative;
use codeilus_core::types::NarrativeKind;
use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ParsedFile;

pub struct NarrativeGenerator {
    // holds LLM client reference
}

impl NarrativeGenerator {
    pub fn new() -> Self;

    /// Generate all narratives for the repo.
    /// Order: Overview, Architecture, ReadingOrder, ExtensionGuide,
    ///        ContributionGuide, WhyTrending, then per-community ModuleSummary.
    /// SymbolExplanation is on-demand only (not pre-generated for all symbols).
    pub async fn generate_all(
        &self,
        graph: &KnowledgeGraph,
        parsed_files: &[ParsedFile],
        repo_path: &std::path::Path,
    ) -> CodeilusResult<Vec<Narrative>>;

    /// Generate a single narrative.
    pub async fn generate_one(
        &self,
        kind: NarrativeKind,
        graph: &KnowledgeGraph,
        target_id: Option<i64>,
    ) -> CodeilusResult<Narrative>;

    /// Generate on-demand symbol explanation.
    pub async fn explain_symbol(
        &self,
        symbol_id: i64,
        graph: &KnowledgeGraph,
    ) -> CodeilusResult<Narrative>;
}
```

Generation logic:
1. Check if LLM is available via `codeilus_llm::is_available()`
2. If available: build context, send prompt, store response
3. If unavailable: generate placeholder content, mark `is_placeholder = true`
4. Log each narrative generation with tracing
5. Per-community ModuleSummary: iterate all communities, generate one per community

### 6. `src/lib.rs` — Module entry point

```rust
pub mod generator;
pub mod placeholders;
pub mod prompts;
pub mod types;

pub use generator::NarrativeGenerator;
pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ParsedFile;
use std::path::Path;

pub async fn generate_all_narratives(
    graph: &KnowledgeGraph,
    parsed_files: &[ParsedFile],
    repo_path: &Path,
) -> CodeilusResult<Vec<Narrative>> {
    NarrativeGenerator::new().generate_all(graph, parsed_files, repo_path).await
}
```

### 7. Add to `crates/codeilus-db/src/repos/` — `narrative_repo.rs`

```rust
use codeilus_core::ids::FileId;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeRow {
    pub id: i64,
    pub kind: String,           // NarrativeKind as string
    pub target_id: Option<i64>, // community_id or symbol_id
    pub title: String,
    pub content: String,
    pub is_placeholder: bool,
}

pub struct NarrativeRepo { conn: Arc<Mutex<Connection>> }

impl NarrativeRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn insert(&self, kind: &str, target_id: Option<i64>, title: &str, content: &str, is_placeholder: bool) -> CodeilusResult<i64>;
    pub fn insert_batch(&self, narratives: &[(String, Option<i64>, String, String, bool)]) -> CodeilusResult<Vec<i64>>;
    pub fn get_by_kind(&self, kind: &str) -> CodeilusResult<Option<NarrativeRow>>;
    pub fn get_by_kind_and_target(&self, kind: &str, target_id: i64) -> CodeilusResult<Option<NarrativeRow>>;
    pub fn list(&self) -> CodeilusResult<Vec<NarrativeRow>>;
    pub fn list_by_kind(&self, kind: &str) -> CodeilusResult<Vec<NarrativeRow>>;
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

Update `crates/codeilus-db/src/repos/mod.rs` to include `narrative_repo`.

## Tests

### Test cases:
1. `prompt_template_overview` — get_prompt(Overview) returns non-empty system and user template
2. `prompt_template_all_kinds` — All 8 NarrativeKind variants have templates
3. `prompt_template_has_context_placeholder` — All user templates contain `{context}`
4. `placeholder_overview` — placeholder_for(Overview) mentions file count
5. `placeholder_reading_order` — placeholder_for(ReadingOrder) lists entry point files
6. `placeholder_module_summary` — placeholder_for(ModuleSummary) lists members
7. `generator_placeholder_mode` — When LLM unavailable, generate_all returns placeholders with is_placeholder=true
8. `generator_all_kinds_covered` — generate_all produces at least 6 narratives (overview + architecture + reading_order + extension + contribution + why_trending) plus per-community
9. `narrative_content_not_empty` — Even placeholders have non-empty content

### DB repo tests:
10. `narrative_repo_insert_and_get` — Insert narrative, get by kind
11. `narrative_repo_get_by_kind_and_target` — Insert module summary for community 1, retrieve by kind+target
12. `narrative_repo_list_by_kind` — Multiple module summaries → list_by_kind("module_summary") returns all

Note: Tests requiring actual LLM calls should use `#[ignore]` and be treated as integration tests.

## Acceptance Criteria

- [ ] `cargo test -p codeilus-narrate` — all unit tests pass
- [ ] `cargo clippy -p codeilus-narrate` — zero warnings
- [ ] `cargo test -p codeilus-db` — all tests pass (including narrative repo)
- [ ] All 8 narrative kinds have prompt templates
- [ ] All prompt templates reference PocketFlow patterns (analogies, beginner-friendly)
- [ ] Placeholder mode produces useful data-driven content (not just "unavailable")
- [ ] ReadingOrder placeholder uses actual entry point scores (no LLM needed)
- [ ] ModuleSummary generated per community
- [ ] SymbolExplanation available on-demand
- [ ] is_placeholder flag correctly set

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- `crates/codeilus-graph/` (wave 2)
- `crates/codeilus-parse/` (wave 1)
- `crates/codeilus-llm/` (wave 4 — parallel, read-only from narrate's perspective)
- `crates/codeilus-db/src/pool.rs`, `migrations.rs`, `batch_writer.rs`
- Existing repo files in `crates/codeilus-db/src/repos/`
- `migrations/0001_init.sql`
- Any files outside `crates/codeilus-narrate/` and the new DB repo file

---

## Report

> **Agent: fill this section when done.**

### Status: pending

### Files Created/Modified:
<!-- list all files you created/modified -->

### Tests:
<!-- paste `cargo test -p codeilus-narrate` output -->

### Clippy:
<!-- paste `cargo clippy -p codeilus-narrate` output -->

### Issues / Blockers:
<!-- any problems encountered -->

### Notes:
<!-- anything the next wave needs to know -->
