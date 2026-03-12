# Task: Diagram Generator

> **Crate:** `crates/codeilus-diagram/`
> **Wave:** 3 (parallel with metrics, analyze)
> **Depends on:** codeilus-core (done), codeilus-graph (wave 2), codeilus-parse (wave 1)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 9, Sprint 4 diagram deliverables
- `crates/codeilus-graph/src/types.rs` — KnowledgeGraph, Community, GraphNode, GraphEdge
- `crates/codeilus-parse/src/types.rs` — ParsedFile, ExtractedSymbol
- Reference: `../CodeVisualizer/src/ir/` — FlowchartIR data structures, AST node types, IR → Mermaid conversion
- Reference: `../gitdiagram/` — LLM-enhanced diagram pipeline, Mermaid validation, auto-fix loop
- Reference: `../GitHubTree/` — ASCII file tree rendering, 4 display styles

## Objective

Generate three types of diagrams from graph and parse data:
1. **Architecture diagram**: communities → Mermaid subgraphs with inter-community edges
2. **Flowchart diagrams**: function-level control flow via FlowchartIR → Mermaid
3. **ASCII file tree**: directory tree in 4 styles (default, compact, extended, minimal)

All diagram output is Mermaid syntax (strings) or ASCII text. Include Mermaid validation and escaping.

Public API:
```rust
pub fn generate_architecture(graph: &KnowledgeGraph) -> CodeilusResult<String>
pub fn generate_flowchart(symbol: &ExtractedSymbol, source: &str) -> CodeilusResult<String>
pub fn generate_file_tree(files: &[String], style: TreeStyle) -> String
pub fn validate_mermaid(mermaid: &str) -> ValidationResult
```

## Files to Create/Modify

### 1. Update `crates/codeilus-diagram/Cargo.toml`

```toml
[package]
name = "codeilus-diagram"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 2. `src/types.rs` — Diagram IR types

```rust
use serde::{Serialize, Deserialize};

/// Flowchart intermediate representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowchartIR {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub kind: FlowNodeKind,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowNodeKind {
    Entry,
    Exit,
    Process,
    Decision,
    Loop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,  // "yes", "no", "else", etc.
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeStyle {
    Default,    // ├── file.rs
    Compact,    // | file.rs
    Extended,   // ├── file.rs (123 lines, 5 symbols)
    Minimal,    // file.rs
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}
```

### 3. `src/architecture.rs` — Architecture diagram generator

- Input: `&KnowledgeGraph` with communities, nodes, edges
- Output: Mermaid `graph TD` with subgraphs per community
- Each community becomes a `subgraph Community_N["Label"]`
- Nodes inside subgraphs show symbol name and kind
- Inter-community edges shown as dashed arrows
- Limit to top 50 nodes (by fan-in score) to avoid huge diagrams
- Escape special characters in labels (quotes, brackets, parens)
- Reference: `../gitdiagram/` Mermaid generation patterns

Example output:
```
graph TD
    subgraph C0["Core Parser"]
        n1["parse_file (fn)"]
        n2["Parser (struct)"]
    end
    subgraph C1["HTTP Layer"]
        n3["handle_request (fn)"]
    end
    n1 -.-> n3
```

### 4. `src/flowchart.rs` — Flowchart generator

- Input: `ExtractedSymbol` + source code text of that symbol
- Build `FlowchartIR` from simple heuristic analysis:
  - Entry node = function start
  - Exit node = function end / return statements
  - `if`/`match`/`switch` → Decision node with yes/no edges
  - `for`/`while`/`loop` → Loop node with back-edge
  - Everything else → Process node
- Convert `FlowchartIR` → Mermaid `flowchart TD` syntax
- Reference: `../CodeVisualizer/src/ir/` for FlowchartIR patterns and AST → IR conversion

Mermaid node shapes by kind:
- Entry: `([label])` (stadium)
- Exit: `([label])` (stadium)
- Process: `[label]` (rectangle)
- Decision: `{label}` (diamond)
- Loop: `[[label]]` (subroutine)

### 5. `src/file_tree.rs` — ASCII file tree

- Input: `&[String]` (sorted file paths)
- Output: ASCII tree string
- Sort: directories first, then files, alphabetical within each
- 4 styles from `TreeStyle` enum
- Handle nested directories with proper indentation
- Reference: `../GitHubTree/` for tree styles and sorting

Example (Default style):
```
src/
├── main.rs
├── lib.rs
├── parser/
│   ├── mod.rs
│   ├── python.rs
│   └── rust.rs
└── utils.rs
```

### 6. `src/mermaid.rs` — Mermaid validation and escaping

- `validate_mermaid(input: &str) -> ValidationResult`:
  - Check for balanced brackets, quotes, parentheses
  - Check for valid graph/flowchart/subgraph keywords
  - Check node IDs don't contain special characters
  - Check edges use valid syntax (`-->`, `-.->`, `==>`)
- `escape_label(label: &str) -> String`:
  - Escape `"`, `(`, `)`, `[`, `]`, `{`, `}`, `<`, `>`
  - Replace newlines with `<br/>`
  - Truncate labels >60 chars with "..."
- `sanitize_node_id(id: &str) -> String`:
  - Replace non-alphanumeric chars with `_`
  - Ensure starts with letter

### 7. `src/lib.rs` — Module entry point

```rust
pub mod architecture;
pub mod file_tree;
pub mod flowchart;
pub mod mermaid;
pub mod types;

pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ExtractedSymbol;

pub fn generate_architecture(graph: &KnowledgeGraph) -> CodeilusResult<String> {
    architecture::generate(graph)
}

pub fn generate_flowchart(symbol: &ExtractedSymbol, source: &str) -> CodeilusResult<String> {
    flowchart::generate(symbol, source)
}

pub fn generate_file_tree(files: &[String], style: TreeStyle) -> String {
    file_tree::generate(files, style)
}

pub fn validate_mermaid(mermaid: &str) -> ValidationResult {
    mermaid::validate(mermaid)
}
```

## Tests

### Test cases:
1. `architecture_two_communities` — Graph with 2 communities → Mermaid with 2 subgraphs
2. `architecture_inter_community_edges` — Edge between communities → dashed arrow in output
3. `architecture_label_escaping` — Node name with `"quotes"` → escaped in output
4. `architecture_node_limit` — Graph with 100 nodes → output limited to 50
5. `flowchart_simple_function` — Linear function (no branches) → Entry → Process → Exit
6. `flowchart_if_else` — Function with if/else → Decision node with yes/no edges
7. `flowchart_for_loop` — Function with for loop → Loop node with back-edge
8. `flowchart_nested` — Nested if inside loop → correct nesting
9. `file_tree_default_style` — 5 files in 2 dirs → correct ASCII tree with `├──` and `└──`
10. `file_tree_compact_style` — Same files → compact style output
11. `file_tree_dirs_first` — Directories sorted before files
12. `file_tree_nested_dirs` — 3-level deep nesting → correct indentation
13. `mermaid_valid` — Well-formed Mermaid → valid=true, empty errors
14. `mermaid_unbalanced_brackets` — Missing closing bracket → valid=false
15. `mermaid_escape_label` — Label with special chars → properly escaped
16. `mermaid_sanitize_id` — ID with spaces/special chars → sanitized to alphanumeric

## Acceptance Criteria

- [ ] `cargo test -p codeilus-diagram` — all tests pass
- [ ] `cargo clippy -p codeilus-diagram` — zero warnings
- [ ] `generate_architecture` produces valid Mermaid `graph TD` with subgraphs
- [ ] `generate_flowchart` produces valid Mermaid `flowchart TD` from source code
- [ ] `generate_file_tree` produces correct ASCII tree in 4 styles
- [ ] Mermaid validation catches common syntax errors
- [ ] Label escaping prevents Mermaid rendering failures
- [ ] Node limit prevents oversized diagrams

## Do NOT Touch
- `crates/codeilus-core/` (read-only)
- `crates/codeilus-parse/` (wave 1)
- `crates/codeilus-graph/` (wave 2)
- Any DB files — this crate has no DB dependency
- `Cargo.toml` at workspace root
- Any files outside `crates/codeilus-diagram/`

---

## Report

> **Agent: fill this section when done.**

### Status: pending

### Files Created/Modified:
<!-- list all files you created/modified -->

### Tests:
<!-- paste `cargo test -p codeilus-diagram` output -->

### Clippy:
<!-- paste `cargo clippy -p codeilus-diagram` output -->

### Issues / Blockers:
<!-- any problems encountered -->

### Notes:
<!-- anything the next wave needs to know -->
