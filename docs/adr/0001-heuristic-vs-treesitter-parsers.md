# ADR-0001: Replace Heuristic Parsers with Tree-sitter

**Status:** Proposed
**Date:** 2026-03-12
**Sprint:** 0 → 1 transition
**Decider:** Human (pending)

## Context

The current `codeilus-parse` crate has 6 language parsers (Python, TypeScript, JavaScript, Rust, Go, Java) implemented as line-by-line string matching heuristics. They extract function/class names and imports via regex-like patterns.

**What they can do:**
- Detect `def foo():` as a Python function
- Detect `import os` as an import
- Count SLOC

**What they cannot do:**
- Extract function calls (who calls whom)
- Extract heritage (extends/implements)
- Handle multi-line signatures
- Parse nested scopes correctly
- Understand language semantics (decorators, generics, closures)

Every downstream crate depends on rich parsed data:
- `codeilus-graph` needs calls + heritage to build the knowledge graph
- `codeilus-metrics` needs accurate symbol boundaries for complexity
- `codeilus-analyze` needs call patterns for anti-pattern detection
- `codeilus-learn` needs the graph to generate curricula

## Decision

Replace all 6 heuristic parsers with tree-sitter grammars. Tree-sitter provides:
- Real AST parsing (not line-by-line)
- Pre-built grammars for all 6 languages
- Query language for pattern extraction (S-expression queries)
- Incremental parsing (for future re-analysis)
- Battle-tested in VS Code, Neovim, GitHub

## Alternatives Considered

1. **Keep heuristic parsers, improve them** — Rejected. Would need months of work per language and still wouldn't match tree-sitter quality. Every edge case requires custom handling.

2. **Use syn (Rust only) + language-specific parsers** — Rejected. Different parser per language means 6x the maintenance. Tree-sitter provides a unified API.

3. **Use LSP servers** — Rejected. Requires installing language toolchains. Violates "single binary, zero deps" principle.

## Consequences

**Positive:**
- Accurate AST extraction for all 6 languages
- Call and heritage extraction become possible
- Single API for all languages (tree-sitter queries)
- Future language support is adding a grammar + queries

**Negative:**
- Binary size increases (~5-10MB for grammar .so files with `bundled` feature)
- Build time increases (tree-sitter grammars compile C code)
- Tree-sitter query strings require learning S-expression syntax
- Some grammars may lag behind latest language features
