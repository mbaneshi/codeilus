# Task: Tree-sitter Parsing Engine

> **Crate:** `crates/codeilus-parse/`
> **Wave:** 1 (parallel with db-repos, frontend)
> **Depends on:** codeilus-core (done)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `NORTH_STAR.md` — section 6.2 (codeilus-parse deep dive)
- `crates/codeilus-core/src/types.rs` — Language, SymbolKind enums you must use
- `crates/codeilus-core/src/error.rs` — CodeilusError, CodeilusResult
- Reference: `../GitNexus/src/core/ingestion/` — parsing pipeline patterns to port

## Objective

Build the parsing engine that walks a repository, parses source files with tree-sitter, and extracts symbols, imports, calls, and heritage relationships.

Public API: `pub fn parse_repository(path: &Path) -> CodeilusResult<Vec<ParsedFile>>`

## Files to Create

### 1. Update `crates/codeilus-parse/Cargo.toml`

```toml
[package]
name = "codeilus-parse"
version = "0.1.0"
edition = "2021"

[dependencies]
codeilus-core = { path = "../codeilus-core" }
tree-sitter = "0.24"
tree-sitter-python = "0.23"
tree-sitter-typescript = "0.23"
tree-sitter-javascript = "0.23"
tree-sitter-rust = "0.23"
tree-sitter-go = "0.23"
tree-sitter-java = "0.23"
ignore = "0.4"
rayon = "1"
tracing = { workspace = true }
serde = { workspace = true }
serde_json = { workspace = true }
```

### 2. `src/types.rs` — Output types

```rust
use codeilus_core::types::{Language, SymbolKind};
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedFile {
    pub path: String,
    pub language: Language,
    pub sloc: usize,
    pub symbols: Vec<ExtractedSymbol>,
    pub imports: Vec<ExtractedImport>,
    pub calls: Vec<ExtractedCall>,
    pub heritage: Vec<ExtractedHeritage>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedSymbol {
    pub name: String,
    pub kind: SymbolKind,
    pub start_line: usize,
    pub end_line: usize,
    pub signature: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedImport {
    pub source: String,
    pub names: Vec<String>,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedCall {
    pub caller: String,
    pub callee: String,
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractedHeritage {
    pub child: String,
    pub parent: String,
    pub kind: HeritageKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HeritageKind {
    Extends,
    Implements,
}
```

### 3. `src/walker.rs` — Filesystem walker

- Use `ignore::WalkBuilder` to respect `.gitignore`
- Filter files by known extensions (use `Language::from_extension`)
- Enforce 20MB total byte budget (skip files beyond budget)
- Return `Vec<(PathBuf, Language)>`

### 4. `src/language.rs` — Parser factory

- Map `Language` enum to tree-sitter grammar
- `fn create_parser(lang: Language) -> Result<tree_sitter::Parser>`
- Only support 6 languages initially: Python, TypeScript, JavaScript, Rust, Go, Java

### 5. `src/extractor.rs` — Symbol extraction

- Given a parsed `tree_sitter::Tree` + source bytes + language, extract all data
- Use query strings from `src/queries/*.rs`
- `fn extract(tree: &Tree, source: &[u8], lang: Language) -> ParsedFile`

### 6. `src/queries/mod.rs` — Query module

```rust
pub mod python;
pub mod typescript;
pub mod rust_lang;
pub mod go;
pub mod java;

pub fn get_queries(lang: Language) -> &'static LanguageQueries { ... }

pub struct LanguageQueries {
    pub definitions: &'static str,    // tree-sitter query for function/class defs
    pub imports: &'static str,        // tree-sitter query for import statements
    pub calls: &'static str,          // tree-sitter query for function calls
    pub heritage: &'static str,       // tree-sitter query for extends/implements
}
```

### 7. `src/queries/python.rs`

Tree-sitter queries for Python:
- Definitions: `function_definition`, `class_definition`
- Imports: `import_statement`, `import_from_statement`
- Calls: `call` expressions
- Heritage: class bases in `class_definition`

### 8. `src/queries/typescript.rs`

TS/JS shared queries:
- Definitions: `function_declaration`, `class_declaration`, `method_definition`, `arrow_function` (when assigned)
- Imports: `import_statement`
- Calls: `call_expression`
- Heritage: `extends_clause`, `implements_clause`

### 9. `src/queries/rust_lang.rs`

- Definitions: `function_item`, `struct_item`, `enum_item`, `trait_item`, `impl_item`
- Imports: `use_declaration`
- Calls: `call_expression`
- Heritage: `impl_item` with trait (e.g., `impl Trait for Struct`)

### 10. `src/queries/go.rs`

- Definitions: `function_declaration`, `method_declaration`, `type_declaration`
- Imports: `import_declaration`
- Calls: `call_expression`
- Heritage: interface embedding

### 11. `src/queries/java.rs`

- Definitions: `class_declaration`, `method_declaration`, `interface_declaration`
- Imports: `import_declaration`
- Calls: `method_invocation`
- Heritage: `extends`, `implements`

### 12. `src/resolver.rs` — Import resolution

- Given an import source path + the file's location, resolve to a file path in the repo
- Python: relative imports (`from . import x`), absolute (`import os.path`)
- TypeScript: resolve `./` relative, `@/` aliases, bare imports
- Rust: `use crate::`, `use super::`, `mod` declarations
- Go: package path resolution
- Java: package → directory path
- Returns `Option<String>` (resolved path or None for external)

### 13. `src/lib.rs` — Main entry point

```rust
pub mod extractor;
pub mod language;
pub mod queries;
pub mod resolver;
pub mod types;
pub mod walker;

pub use types::*;

use codeilus_core::CodeilusResult;
use rayon::prelude::*;
use std::path::Path;

/// Parse all source files in a repository.
pub fn parse_repository(path: &Path) -> CodeilusResult<Vec<ParsedFile>> {
    let files = walker::walk(path)?;
    let results: Vec<ParsedFile> = files
        .par_iter()
        .filter_map(|(path, lang)| {
            match parse_single_file(path, *lang) {
                Ok(pf) => Some(pf),
                Err(e) => {
                    tracing::warn!(path = %path.display(), error = %e, "failed to parse");
                    None
                }
            }
        })
        .collect();
    Ok(results)
}
```

## Tests

Create `crates/codeilus-parse/tests/fixtures/` with small sample files:

- `tests/fixtures/sample.py`:
```python
import os
from pathlib import Path

class FileReader:
    def __init__(self, path):
        self.path = path

    def read(self):
        return Path(self.path).read_text()

def process(reader):
    content = reader.read()
    return content.upper()
```

- `tests/fixtures/sample.ts`:
```typescript
import { readFile } from 'fs/promises';

interface Reader {
  read(): Promise<string>;
}

class FileReader implements Reader {
  constructor(private path: string) {}

  async read(): Promise<string> {
    return readFile(this.path, 'utf-8');
  }
}

export function process(reader: Reader): Promise<string> {
  return reader.read();
}
```

- `tests/fixtures/sample.rs`:
```rust
use std::fs;

pub struct Config {
    pub path: String,
}

impl Config {
    pub fn load(path: &str) -> Self {
        Self { path: path.to_string() }
    }

    pub fn read(&self) -> String {
        fs::read_to_string(&self.path).unwrap_or_default()
    }
}

pub fn process(config: &Config) -> String {
    config.read().to_uppercase()
}
```

### Test cases:
1. `parse_repository` finds all 3 fixture files
2. Python: extracts `FileReader` (class), `__init__` (method), `read` (method), `process` (function)
3. Python: extracts `import os` and `from pathlib import Path`
4. TypeScript: extracts `Reader` (interface), `FileReader` (class), `process` (function)
5. TypeScript: extracts `implements Reader` heritage
6. Rust: extracts `Config` (struct), `load` (method), `read` (method), `process` (function)
7. SLOC counts are reasonable (non-zero, ≤ file line count)

## Acceptance Criteria

- [ ] `cargo test -p codeilus-parse` — all tests pass
- [ ] `cargo clippy -p codeilus-parse` — zero warnings
- [ ] `parse_repository("tests/fixtures/")` returns 3 ParsedFiles
- [ ] Each ParsedFile has correct language, non-zero symbols, non-zero SLOC
- [ ] At least Python + TypeScript + Rust extract function/class definitions correctly

## Do NOT Touch
- Any files outside `crates/codeilus-parse/`
- `Cargo.toml` at workspace root
- Any other crate

---

## Report

### Status: complete

### Files Created:
- `src/extractor.rs` — Tree-sitter symbol/import/call/heritage extraction engine
- `src/language.rs` — Updated: now includes `create_parser()` factory using tree-sitter grammars
- `src/resolver.rs` — Import path resolution for Python, TS/JS, Rust, Go, Java
- `src/queries/mod.rs` — Query module with `get_queries(lang)` dispatcher
- `src/queries/python.rs` — Tree-sitter queries for Python definitions, imports, calls, heritage
- `src/queries/typescript.rs` — Tree-sitter queries for TS/JS (shared grammar patterns)
- `src/queries/rust_lang.rs` — Tree-sitter queries for Rust
- `src/queries/go.rs` — Tree-sitter queries for Go
- `src/queries/java.rs` — Tree-sitter queries for Java
- `src/model.rs` — Updated: added `sloc` field to `ParsedFile`, `line` field to `Import`, added Serialize/Deserialize derives
- `src/lib.rs` — Updated: added `queries`, `resolver`, `extractor` modules; exports `create_parser`
- `src/parser/*.rs` — All 6 parsers rewritten from line-based heuristics to tree-sitter
- `Cargo.toml` — Added tree-sitter + grammar crates, streaming-iterator, serde, serde_json
- `tests/fixtures/sample.py` — Python fixture
- `tests/fixtures/sample.ts` — TypeScript fixture
- `tests/fixtures/sample.rs` — Rust fixture
- `tests/basic_parsing.rs` — 10 test cases covering all acceptance criteria

### Tests:
```
running 10 tests
test detect_language_by_extension ... ok
test parse_repository_with_tempdir ... ok
test typescript_extracts_heritage ... ok
test python_extracts_symbols ... ok
test typescript_extracts_symbols ... ok
test rust_extracts_symbols ... ok
test each_parsed_file_has_correct_language_and_nonempty_symbols ... ok
test python_extracts_imports ... ok
test sloc_counts_are_reasonable ... ok
test parse_repository_finds_all_fixture_files ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

### Clippy:
```
Finished `dev` profile [unoptimized + debuginfo] target(s) — zero warnings
```

### Issues / Blockers:
None.

### Notes:
- tree-sitter 0.24 uses `StreamingIterator` (not std `Iterator`) for query matches/captures — the `streaming-iterator` crate is required
- The `Import` model now has a `line` field (breaking change for db-repos agent if they depend on old `Import` struct)
- `ParsedFile` now has a `sloc` field (non-empty line count)
- All parsers delegate to the shared `extractor::extract()` function — adding a new language only requires a new query file + grammar dep
- Heritage queries need to anchor at the class/struct declaration to capture both `@child` (class name) and `@parent` (superclass/interface)
- `resolver.rs` does basic local-only resolution; returns `None` for external/third-party imports
