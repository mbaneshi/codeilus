# Task: Anti-Pattern Analyzer

> **Crate:** `crates/codeilus-analyze/`
> **Wave:** 3 (parallel with metrics, diagram)
> **Depends on:** codeilus-core (done), codeilus-parse (wave 1), codeilus-graph (wave 2), codeilus-db (wave 1+2)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 9, Sprint 3 analysis deliverables
- `crates/codeilus-core/src/types.rs` — SymbolKind
- `crates/codeilus-core/src/ids.rs` — FileId, SymbolId
- `crates/codeilus-parse/src/types.rs` — ParsedFile, ExtractedSymbol
- `crates/codeilus-graph/src/types.rs` — KnowledgeGraph (petgraph DiGraph)
- Reference: `../GitVizz/` — anti-pattern detection patterns, severity levels, heuristics

## Objective

Detect anti-patterns, security hotspots, and code quality issues. Five detectors: god class, long method, circular dependencies, security hotspots, and test gap detection. Each finding has a severity level (info/warning/error) and actionable suggestion. Persist findings to the `patterns` table.

Public API:
```rust
pub fn analyze(
    parsed_files: &[ParsedFile],
    graph: &KnowledgeGraph,
) -> CodeilusResult<Vec<PatternFinding>>
```

## Files to Create/Modify

### 1. Update `crates/codeilus-analyze/Cargo.toml`

```toml
[package]
name = "codeilus-analyze"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
codeilus-db = { path = "../codeilus-db" }
petgraph = "0.6"
regex = "1"
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 2. `src/types.rs` — Finding types

```rust
use codeilus_core::ids::{FileId, SymbolId};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Severity {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PatternKind {
    GodClass,
    LongMethod,
    CircularDependency,
    SecurityHotspot,
    TestGap,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternFinding {
    pub kind: PatternKind,
    pub severity: Severity,
    pub file_id: Option<FileId>,
    pub symbol_id: Option<SymbolId>,
    pub file_path: String,
    pub line: Option<usize>,
    pub message: String,
    pub suggestion: String,
}
```

### 3. `src/god_class.rs` — God class detector

- A class/struct is a "god class" if it has **>20 methods**
- Scan ParsedFiles for symbols where kind == Class/Struct
- Count methods belonging to each class (symbols within the class's line range, kind == Method)
- Severity:
  - 21-30 methods → Warning
  - 31+ methods → Error
- Suggestion: "Consider splitting into smaller classes with single responsibilities"
- Reference: `../GitVizz/` god class detection

### 4. `src/long_method.rs` — Long method detector

- A method/function is "long" if it has **>50 lines of code** (end_line - start_line)
- Severity:
  - 51-100 lines → Info
  - 101-200 lines → Warning
  - 201+ lines → Error
- Suggestion: "Extract helper methods to improve readability"

### 5. `src/circular_deps.rs` — Circular dependency detector

- Run DFS cycle detection on the dependency graph from KnowledgeGraph
- Use petgraph's `algo::is_cyclic_directed()` or manual DFS with back-edge detection
- Report each cycle found as a finding
- Group by the smallest cycle (avoid duplicate reports for overlapping cycles)
- Severity: always Warning
- Message: list the files in the cycle (e.g., "Circular: A → B → C → A")
- Suggestion: "Break the cycle by extracting shared types into a common module"

### 6. `src/security.rs` — Security hotspot detector

- Regex-based scanning of source file content for dangerous patterns:
  - `eval(` / `exec(` → "Dynamic code execution"
  - `SQL` + string concatenation / f-string → "Potential SQL injection"
  - Patterns matching secrets: `password\s*=\s*["']`, `api_key\s*=\s*["']`, `secret\s*=\s*["']` → "Hardcoded secret"
  - `subprocess` / `os.system` / `child_process` → "Command injection risk"
  - `innerHTML` / `dangerouslySetInnerHTML` → "XSS risk"
- Severity: always Error for secrets, Warning for others
- Suggestion: context-specific (e.g., "Use parameterized queries instead of string concatenation")
- Note: this requires reading file content — accept `&[(String, String)]` (path, content) or read files from disk

### 7. `src/test_gap.rs` — Test gap detector

- Identify files that likely need tests but don't have them:
  - For each source file, check if a corresponding test file exists (e.g., `foo.py` → `test_foo.py` or `foo_test.py` or `tests/foo.py`)
  - Also check: `*.test.ts`, `*.spec.ts`, `*_test.go`, `*_test.rs`
- Only flag files with >5 symbols (trivial files don't need tests)
- Severity: Info
- Suggestion: "Add tests for this file — it has N public symbols"

### 8. `src/lib.rs` — Module entry point

```rust
pub mod circular_deps;
pub mod god_class;
pub mod long_method;
pub mod security;
pub mod test_gap;
pub mod types;

pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ParsedFile;

/// Run all analyzers and return combined findings.
pub fn analyze(
    parsed_files: &[ParsedFile],
    graph: &KnowledgeGraph,
) -> CodeilusResult<Vec<PatternFinding>> {
    let mut findings = Vec::new();
    findings.extend(god_class::detect(parsed_files)?);
    findings.extend(long_method::detect(parsed_files)?);
    findings.extend(circular_deps::detect(graph)?);
    findings.extend(security::detect(parsed_files)?);
    findings.extend(test_gap::detect(parsed_files)?);
    Ok(findings)
}
```

### 9. Add to `crates/codeilus-db/src/repos/` — `pattern_repo.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternRow {
    pub id: i64,
    pub kind: String,
    pub severity: String,
    pub file_id: Option<i64>,
    pub symbol_id: Option<i64>,
    pub file_path: String,
    pub line: Option<i64>,
    pub message: String,
    pub suggestion: String,
}

pub struct PatternRepo { conn: Arc<Mutex<Connection>> }

impl PatternRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;
    pub fn insert(&self, finding: &PatternRow) -> CodeilusResult<i64>;
    pub fn insert_batch(&self, findings: &[PatternRow]) -> CodeilusResult<Vec<i64>>;
    pub fn list(&self) -> CodeilusResult<Vec<PatternRow>>;
    pub fn list_by_severity(&self, severity: &str) -> CodeilusResult<Vec<PatternRow>>;
    pub fn list_by_kind(&self, kind: &str) -> CodeilusResult<Vec<PatternRow>>;
    pub fn list_by_file(&self, file_id: i64) -> CodeilusResult<Vec<PatternRow>>;
    pub fn count_by_severity(&self) -> CodeilusResult<Vec<(String, usize)>>;
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

Update `crates/codeilus-db/src/repos/mod.rs` to include `pattern_repo`.

## Tests

### Test cases:
1. `god_class_detected` — Class with 25 methods → Warning finding
2. `god_class_not_triggered` — Class with 10 methods → no finding
3. `god_class_severe` — Class with 35 methods → Error finding
4. `long_method_50_lines` — Method with exactly 50 lines → no finding (must exceed)
5. `long_method_100_lines` — Method with 100 lines → Warning
6. `long_method_250_lines` — Method with 250 lines → Error
7. `circular_dep_simple` — A→B→A cycle detected
8. `circular_dep_three_node` — A→B→C→A cycle detected
9. `circular_dep_none` — DAG with no cycles → no findings
10. `security_eval` — File containing `eval(user_input)` → Warning
11. `security_hardcoded_secret` — File with `password = "abc123"` → Error
12. `security_sql_injection` — File with `f"SELECT * FROM {table}"` → Warning
13. `security_clean_file` — Normal code → no findings
14. `test_gap_missing` — `src/parser.py` with 10 symbols, no test file → Info
15. `test_gap_covered` — `src/parser.py` exists with `tests/test_parser.py` → no finding
16. `analyze_integration` — Full analyze() returns combined findings from all detectors

### DB repo tests:
17. `pattern_repo_insert_and_list` — Insert findings, list all
18. `pattern_repo_filter_by_severity` — Filter by "error" returns only errors

## Acceptance Criteria

- [ ] `cargo test -p codeilus-analyze` — all tests pass
- [ ] `cargo clippy -p codeilus-analyze` — zero warnings
- [ ] `cargo test -p codeilus-db` — all tests pass (including new repo tests)
- [ ] God class: detects classes with >20 methods
- [ ] Long method: detects methods with >50 LOC
- [ ] Circular deps: finds cycles via DFS on petgraph
- [ ] Security: regex catches eval, exec, hardcoded secrets, SQL injection, XSS
- [ ] Test gap: identifies source files without corresponding test files
- [ ] All findings have severity (info/warning/error) and actionable suggestion
- [ ] `analyze()` combines all detector results

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- `crates/codeilus-parse/` (wave 1)
- `crates/codeilus-graph/` (wave 2)
- `crates/codeilus-db/src/pool.rs`, `migrations.rs`, `batch_writer.rs`
- Existing repo files in `crates/codeilus-db/src/repos/`
- `migrations/0001_init.sql`
- Any files outside `crates/codeilus-analyze/` and the new DB repo file

---

## Report

> **Agent: fill this section when done.**

### Status: pending

### Files Created/Modified:
<!-- list all files you created/modified -->

### Tests:
<!-- paste `cargo test -p codeilus-analyze` output -->

### Clippy:
<!-- paste `cargo clippy -p codeilus-analyze` output -->

### Issues / Blockers:
<!-- any problems encountered -->

### Notes:
<!-- anything the next wave needs to know -->
