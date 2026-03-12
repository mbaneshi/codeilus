# Task: Code Metrics Engine

> **Crate:** `crates/codeilus-metrics/`
> **Wave:** 3 (parallel with analyze, diagram)
> **Depends on:** codeilus-core (done), codeilus-parse (wave 1), codeilus-graph (wave 2), codeilus-db (wave 1+2)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 6, Sprint 3 metrics deliverables
- `crates/codeilus-core/src/types.rs` — Language, SymbolKind
- `crates/codeilus-core/src/ids.rs` — FileId, SymbolId, CommunityId
- `crates/codeilus-parse/src/types.rs` — ParsedFile (has sloc, symbols)
- `crates/codeilus-graph/src/types.rs` — KnowledgeGraph, Community
- `crates/codeilus-db/src/repos/edge_repo.rs` — EdgeRepo for fan-in/out counts
- Reference: `../emerge/emerge/metrics/` — SLOC, fan-in/out, modularity, TF-IDF, heatmap patterns

## Objective

Compute code metrics: SLOC breakdown, fan-in/fan-out per symbol, cyclomatic complexity estimate, Louvain modularity Q-score, TF-IDF keywords per community, git churn/contributors via git2-rs, and heatmap scoring. Persist all metrics to the `file_metrics` table.

Public API:
```rust
pub fn compute_metrics(
    parsed_files: &[ParsedFile],
    graph: &KnowledgeGraph,
    repo_path: &Path,
    db: &MetricsDb,
) -> CodeilusResult<MetricsReport>
```

## Files to Create/Modify

### 1. Update `crates/codeilus-metrics/Cargo.toml`

```toml
[package]
name = "codeilus-metrics"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
codeilus-db = { path = "../codeilus-db" }
git2 = "0.19"
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 2. `src/types.rs` — Metrics types

```rust
use codeilus_core::ids::{FileId, SymbolId, CommunityId};
use serde::{Serialize, Deserialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsReport {
    pub file_metrics: Vec<FileMetrics>,
    pub symbol_metrics: Vec<SymbolMetrics>,
    pub community_metrics: Vec<CommunityMetrics>,
    pub repo_metrics: RepoMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    pub file_id: FileId,
    pub path: String,
    pub sloc: usize,
    pub complexity: f64,
    pub churn: usize,           // number of git commits touching this file
    pub contributors: usize,    // unique authors
    pub heatmap_score: f64,     // composite hotspot score
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolMetrics {
    pub symbol_id: SymbolId,
    pub fan_in: usize,          // number of incoming edges (callers)
    pub fan_out: usize,         // number of outgoing edges (callees)
    pub complexity: f64,        // estimated cyclomatic complexity
    pub loc: usize,             // lines of code (end_line - start_line)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityMetrics {
    pub community_id: CommunityId,
    pub modularity_q: f64,           // Louvain Q-score for this community
    pub keywords: Vec<(String, f64)>,// TF-IDF top keywords
    pub total_sloc: usize,
    pub member_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMetrics {
    pub total_files: usize,
    pub total_sloc: usize,
    pub total_symbols: usize,
    pub language_breakdown: HashMap<String, usize>,  // language → SLOC
    pub avg_complexity: f64,
    pub modularity_q: f64,           // global modularity Q-score
}
```

### 3. `src/sloc.rs` — SLOC computation

- Count non-blank, non-comment lines per file
- Language-aware comment detection (# for Python, // and /* */ for C-family, -- for SQL)
- Aggregate language breakdown for RepoMetrics
- Reference: `../emerge/emerge/metrics/` SLOC counting

### 4. `src/fan.rs` — Fan-in / fan-out

- Fan-in: count incoming CALLS edges to each symbol
- Fan-out: count outgoing CALLS edges from each symbol
- Use EdgeRepo or KnowledgeGraph in-memory edges
- Return `HashMap<SymbolId, (usize, usize)>` (fan_in, fan_out)

### 5. `src/complexity.rs` — Cyclomatic complexity estimate

- Estimate complexity from parsed symbol data:
  - Base complexity = 1
  - +1 for each branch keyword in the symbol's line range (if, else, elif, match, case, for, while, try, catch, &&, ||)
  - This is an approximation — no full AST walk needed
- If source text is not available, estimate from LOC: `1 + (loc / 10)`
- Return `HashMap<SymbolId, f64>`

### 6. `src/modularity.rs` — Modularity Q-score

- Compute Louvain modularity Q-score from community assignments in KnowledgeGraph
- Q = (1/2m) * Σ [A_ij - (k_i * k_j)/(2m)] * δ(c_i, c_j)
  - A_ij = adjacency matrix
  - k_i = degree of node i
  - m = total edges
  - δ = 1 if same community
- Return global Q-score and per-community Q contribution

### 7. `src/tfidf.rs` — TF-IDF keywords per community

- Extract tokens from symbol names in each community (split camelCase, snake_case)
- Compute TF-IDF: TF = frequency in community, IDF = log(N / df)
- Return top 10 keywords per community sorted by TF-IDF score
- Reference: `../emerge/emerge/metrics/` TF-IDF

### 8. `src/git.rs` — Git metrics via git2-rs

- Open repo with `git2::Repository::open(repo_path)`
- Walk commit log (limit to last 1000 commits for performance)
- Per file: count commits (churn) and unique author emails (contributors)
- Graceful fallback: if not a git repo or git2 fails, return empty metrics
- Return `HashMap<String, (usize, usize)>` (path → (churn, contributors))

### 9. `src/heatmap.rs` — Heatmap scoring

- Composite hotspot score per file:
  - `heatmap_score = 0.4 * normalized_complexity + 0.3 * normalized_churn + 0.2 * normalized_size + 0.1 * normalized_fan_in`
- Normalize each component to 0.0-1.0 range (min-max across all files)
- Higher score = hotter (more attention needed)
- Reference: `../emerge/emerge/metrics/` heatmap

### 10. `src/lib.rs` — Module entry point

```rust
pub mod complexity;
pub mod fan;
pub mod git;
pub mod heatmap;
pub mod modularity;
pub mod sloc;
pub mod tfidf;
pub mod types;

pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ParsedFile;
use std::path::Path;

pub fn compute_metrics(
    parsed_files: &[ParsedFile],
    graph: &KnowledgeGraph,
    repo_path: &Path,
) -> CodeilusResult<MetricsReport> { ... }
```

### 11. Add to `crates/codeilus-db/src/repos/` — `file_metrics_repo.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetricsRow {
    pub id: i64,
    pub file_id: FileId,
    pub sloc: i64,
    pub complexity: f64,
    pub churn: i64,
    pub contributors: i64,
    pub heatmap_score: f64,
}

pub struct FileMetricsRepo { conn: Arc<Mutex<Connection>> }

impl FileMetricsRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn insert(&self, file_id: FileId, sloc: i64, complexity: f64, churn: i64, contributors: i64, heatmap_score: f64) -> CodeilusResult<i64>;
    pub fn insert_batch(&self, metrics: &[(FileId, i64, f64, i64, i64, f64)]) -> CodeilusResult<Vec<i64>>;
    pub fn get_by_file(&self, file_id: FileId) -> CodeilusResult<Option<FileMetricsRow>>;
    pub fn list(&self) -> CodeilusResult<Vec<FileMetricsRow>>;
    pub fn list_hotspots(&self, limit: usize) -> CodeilusResult<Vec<FileMetricsRow>>;  // ordered by heatmap_score DESC
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

Update `crates/codeilus-db/src/repos/mod.rs` to include `file_metrics_repo`.

## Tests

### Test cases:
1. `sloc_python` — Python file with comments and blanks → correct SLOC count
2. `sloc_rust` — Rust file with `//` and `/* */` comments → correct SLOC count
3. `fan_in_out` — Graph with A→B, C→B → B fan_in=2, fan_out=0; A fan_out=1
4. `complexity_simple_function` — Function with 2 if branches → complexity ~3
5. `complexity_fallback` — No source text → estimate from LOC
6. `modularity_q_score` — Two perfect clusters → Q close to 0.5
7. `tfidf_keywords` — Community with symbols "parseFile", "parseToken", "readStream" → "parse" is top keyword
8. `tfidf_camel_case_split` — "parseJSONFile" splits to ["parse", "json", "file"]
9. `git_metrics_non_repo` — Non-git directory → returns empty map, no error
10. `heatmap_scoring` — File with high complexity + high churn → highest score
11. `heatmap_normalization` — All scores between 0.0 and 1.0
12. `compute_metrics_integration` — Full pipeline produces MetricsReport with all sections populated

### DB repo tests:
13. `file_metrics_repo_insert_and_get` — Insert metrics, get by file
14. `file_metrics_repo_list_hotspots` — Insert 5 entries, list_hotspots(3) returns top 3 by score

## Acceptance Criteria

- [ ] `cargo test -p codeilus-metrics` — all tests pass
- [ ] `cargo clippy -p codeilus-metrics` — zero warnings
- [ ] `cargo test -p codeilus-db` — all tests pass (including new repo tests)
- [ ] SLOC counting handles comments for Python, Rust, TypeScript, Go, Java
- [ ] Fan-in/out computed from graph edges
- [ ] Cyclomatic complexity estimated per symbol
- [ ] Modularity Q-score computed from community assignments
- [ ] TF-IDF extracts meaningful keywords (camelCase/snake_case splitting)
- [ ] Git metrics gracefully degrade for non-git repos
- [ ] Heatmap scores normalized to 0.0-1.0

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- `crates/codeilus-parse/` (wave 1)
- `crates/codeilus-graph/` (wave 2)
- `crates/codeilus-db/src/pool.rs`, `migrations.rs`, `batch_writer.rs`
- Existing repo files in `crates/codeilus-db/src/repos/`
- `migrations/0001_init.sql`
- Any files outside `crates/codeilus-metrics/` and the new DB repo file

---

## Report

> **Agent: fill this section when done.**

### Status: pending

### Files Created/Modified:
<!-- list all files you created/modified -->

### Tests:
<!-- paste `cargo test -p codeilus-metrics` output -->

### Clippy:
<!-- paste `cargo clippy -p codeilus-metrics` output -->

### Issues / Blockers:
<!-- any problems encountered -->

### Notes:
<!-- anything the next wave needs to know -->
