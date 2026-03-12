# Task: DB Repositories (File, Symbol, Edge)

> **Crate:** `crates/codeilus-db/`
> **Wave:** 1 (parallel with parse, frontend)
> **Depends on:** codeilus-core (done), codeilus-db foundation (done)
> **Status:** pending

---

## Context

Read these files first:
- `CLAUDE.md` — project rules and conventions
- `crates/codeilus-db/src/pool.rs` — DbPool (already implemented)
- `crates/codeilus-db/src/migrations.rs` — Migrator (already implemented)
- `crates/codeilus-db/src/batch_writer.rs` — BatchWriter (already implemented)
- `crates/codeilus-db/src/lib.rs` — current re-exports
- `migrations/0001_init.sql` — full schema (20 tables)
- `crates/codeilus-core/src/ids.rs` — FileId, SymbolId, EdgeId types
- `crates/codeilus-core/src/types.rs` — Language, SymbolKind, EdgeKind, Confidence

## Objective

Implement repository structs for CRUD operations on the `files`, `symbols`, and `edges` tables. These are the foundation all other repos build upon.

## Files to Create/Modify

### 1. `src/repos/mod.rs` — Replace the empty stub

```rust
pub mod file_repo;
pub mod symbol_repo;
pub mod edge_repo;

pub use file_repo::{FileRepo, FileRow};
pub use symbol_repo::{SymbolRepo, SymbolRow};
pub use edge_repo::{EdgeRepo, EdgeRow};
```

### 2. `src/repos/file_repo.rs`

```rust
use codeilus_core::error::{CodeilusError, CodeilusResult};
use codeilus_core::ids::FileId;
use rusqlite::Connection;
use serde::{Serialize, Deserialize};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileRow {
    pub id: FileId,
    pub path: String,
    pub language: Option<String>,
    pub sloc: i64,
    pub last_modified: Option<String>,
}

pub struct FileRepo {
    conn: Arc<Mutex<Connection>>,
}

impl FileRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    /// Insert a single file. Returns the new FileId.
    pub fn insert(&self, path: &str, language: Option<&str>, sloc: i64) -> CodeilusResult<FileId> { ... }

    /// Batch insert files in a transaction. Returns all FileIds.
    pub fn insert_batch(&self, files: &[(String, Option<String>, i64)]) -> CodeilusResult<Vec<FileId>> { ... }

    /// Get a file by ID.
    pub fn get(&self, id: FileId) -> CodeilusResult<FileRow> { ... }

    /// Get a file by path.
    pub fn get_by_path(&self, path: &str) -> CodeilusResult<Option<FileRow>> { ... }

    /// List all files. Optional language filter.
    pub fn list(&self, language: Option<&str>) -> CodeilusResult<Vec<FileRow>> { ... }

    /// Count total files.
    pub fn count(&self) -> CodeilusResult<usize> { ... }

    /// Delete all files (for re-analysis).
    pub fn delete_all(&self) -> CodeilusResult<()> { ... }
}
```

Implementation notes:
- Use `conn.lock().expect("db mutex poisoned")` to get the connection
- Use `conn.execute()` with `rusqlite::params![]` for inserts
- Use `conn.unchecked_transaction()` for batch operations (we hold `MutexGuard`, not `&mut`)
- Use `conn.query_row()` for single-row queries
- Use `conn.prepare()` + `query_map()` for list queries
- Map `rusqlite::Error` to `CodeilusError::Database(Box::new(e))`
- Return `CodeilusError::NotFound(...)` when get() finds nothing

### 3. `src/repos/symbol_repo.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolRow {
    pub id: SymbolId,
    pub file_id: FileId,
    pub name: String,
    pub kind: String,
    pub start_line: i64,
    pub end_line: i64,
    pub signature: Option<String>,
}

pub struct SymbolRepo {
    conn: Arc<Mutex<Connection>>,
}

impl SymbolRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;

    /// Insert a single symbol.
    pub fn insert(&self, file_id: FileId, name: &str, kind: &str,
                  start_line: i64, end_line: i64, signature: Option<&str>) -> CodeilusResult<SymbolId>;

    /// Batch insert symbols in a transaction.
    pub fn insert_batch(&self, symbols: &[(FileId, String, String, i64, i64, Option<String>)]) -> CodeilusResult<Vec<SymbolId>>;

    /// Get symbol by ID.
    pub fn get(&self, id: SymbolId) -> CodeilusResult<SymbolRow>;

    /// List symbols for a file.
    pub fn list_by_file(&self, file_id: FileId) -> CodeilusResult<Vec<SymbolRow>>;

    /// Search symbols by name (exact match).
    pub fn list_by_name(&self, name: &str) -> CodeilusResult<Vec<SymbolRow>>;

    /// Search symbols by name prefix (LIKE 'name%').
    pub fn search(&self, query: &str) -> CodeilusResult<Vec<SymbolRow>>;

    /// Count total symbols.
    pub fn count(&self) -> CodeilusResult<usize>;

    /// Delete all symbols for a file (for re-parse).
    pub fn delete_by_file(&self, file_id: FileId) -> CodeilusResult<()>;
}
```

### 4. `src/repos/edge_repo.rs`

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EdgeRow {
    pub id: EdgeId,
    pub source_id: SymbolId,
    pub target_id: SymbolId,
    pub kind: String,
    pub confidence: f64,
}

pub struct EdgeRepo {
    conn: Arc<Mutex<Connection>>,
}

impl EdgeRepo {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self;

    /// Insert a single edge.
    pub fn insert(&self, source_id: SymbolId, target_id: SymbolId,
                  kind: &str, confidence: f64) -> CodeilusResult<EdgeId>;

    /// Batch insert edges in a transaction.
    pub fn insert_batch(&self, edges: &[(SymbolId, SymbolId, String, f64)]) -> CodeilusResult<Vec<EdgeId>>;

    /// List edges from a symbol (outgoing).
    pub fn list_from(&self, source_id: SymbolId) -> CodeilusResult<Vec<EdgeRow>>;

    /// List edges to a symbol (incoming).
    pub fn list_to(&self, target_id: SymbolId) -> CodeilusResult<Vec<EdgeRow>>;

    /// List all edges of a specific kind.
    pub fn list_by_kind(&self, kind: &str) -> CodeilusResult<Vec<EdgeRow>>;

    /// Count total edges.
    pub fn count(&self) -> CodeilusResult<usize>;

    /// Delete all edges (for re-analysis).
    pub fn delete_all(&self) -> CodeilusResult<()>;
}
```

### 5. Update `src/lib.rs`

```rust
//! Codeilus database layer: pool, migrations, batch writer, and repositories.

pub mod batch_writer;
pub mod migrations;
pub mod pool;
pub mod repos;

pub use batch_writer::BatchWriter;
pub use migrations::Migrator;
pub use pool::DbPool;
pub use repos::{FileRepo, FileRow, SymbolRepo, SymbolRow, EdgeRepo, EdgeRow};
```

## Tests

All tests use in-memory DB:

```rust
fn setup() -> Arc<Mutex<Connection>> {
    let db = DbPool::in_memory().unwrap();
    let conn = db.connection();
    Migrator::new(&conn).apply_pending().unwrap();
    drop(conn);
    db.conn_arc()
}
```

### Required test cases:

**FileRepo:**
1. `insert_and_get` — insert a file, get by ID, verify fields
2. `insert_batch` — insert 3 files, verify count == 3
3. `get_by_path` — insert, then find by path
4. `get_by_path_not_found` — returns None for unknown path
5. `list_all` — insert 3, list returns 3
6. `list_with_language_filter` — insert Python + Rust, filter by "python" returns 1
7. `delete_all` — insert 3, delete all, count == 0

**SymbolRepo:**
8. `insert_and_get` — insert symbol, get by ID
9. `insert_batch` — insert 5 symbols, verify count
10. `list_by_file` — insert symbols for 2 files, filter by file returns correct ones
11. `list_by_name` — search by exact name
12. `search_prefix` — search "proc" matches "process" and "processor"
13. `delete_by_file` — delete symbols for one file, others remain

**EdgeRepo:**
14. `insert_and_list_from` — insert edge, list outgoing from source
15. `insert_batch` — insert 10 edges in batch
16. `list_to` — list incoming edges to a target
17. `list_by_kind` — filter edges by CALLS vs IMPORTS

## Acceptance Criteria

- [ ] `cargo test -p codeilus-db` — all tests pass (including original 0 + new ~17)
- [ ] `cargo clippy -p codeilus-db` — zero warnings
- [ ] FileRepo: insert, get, get_by_path, list (with filter), count, delete_all
- [ ] SymbolRepo: insert, get, list_by_file, list_by_name, search, count, delete_by_file
- [ ] EdgeRepo: insert, list_from, list_to, list_by_kind, count, delete_all
- [ ] All batch operations use transactions

## Do NOT Touch
- `src/pool.rs` (done)
- `src/migrations.rs` (done)
- `src/batch_writer.rs` (done)
- `migrations/0001_init.sql` (done)
- Any files outside `crates/codeilus-db/`

---

## Report

> **Agent: fill this section when done.**

### Status: complete

### Files Created/Modified:
- `src/repos/file_repo.rs` — Created: FileRepo (insert, insert_batch, get, get_by_path, list, count, delete_all) + FileRow
- `src/repos/symbol_repo.rs` — Created: SymbolRepo (insert, insert_batch, get, list_by_file, list_by_name, search, count, delete_by_file) + SymbolRow + NewSymbolBatch
- `src/repos/edge_repo.rs` — Created: EdgeRepo (insert, insert_batch, list_from, list_to, list_by_kind, count, delete_all) + EdgeRow
- `src/repos/mod.rs` — Updated: exports new repo modules alongside existing ones
- `src/lib.rs` — Updated: re-exports FileRepo, FileRow, SymbolRepo, SymbolRow, EdgeRepo, EdgeRow; fixed name collision with old repos
- `tests/repos.rs` — Created: 17 test cases covering all three repos
- `tests/persist_parsed.rs` — Fixed: added missing `sloc` field to ParsedFile (struct changed upstream)

### Tests:
```
running 0 tests (unit)
running 1 test (persist_parsed): ok
running 17 tests (repos): all ok
  file_insert_and_get, file_insert_batch, file_get_by_path, file_get_by_path_not_found,
  file_list_all, file_list_with_language_filter, file_delete_all,
  symbol_insert_and_get, symbol_insert_batch, symbol_list_by_file, symbol_list_by_name,
  symbol_search_prefix, symbol_delete_by_file,
  edge_insert_and_list_from, edge_insert_batch, edge_list_to, edge_list_by_kind
Total: 18 passed, 0 failed
```

### Clippy:
Zero warnings for codeilus-db. (3 warnings in codeilus-parse are unrelated.)

### Issues / Blockers:
- The existing repos (files.rs, symbols.rs, edges.rs) use a different pattern (unit struct + `&Transaction`) from the new repos (`Arc<Mutex<Connection>>`). Both are kept for backward compatibility — the old ones are used by `persist_parsed_files` in lib.rs.

### Notes:
- New repos use `Arc<Mutex<Connection>>` pattern with `unchecked_transaction()` for batch ops (since we hold MutexGuard, not &mut Connection).
- Old repo modules (files, symbols, edges) remain available via `repos::files::FileRepo` etc. for the `persist_parsed_files` method.
- Wave 2 should use the new repos (FileRepo, SymbolRepo, EdgeRepo) — they are re-exported from the crate root.
- `NewSymbolBatch` type alias exists to avoid clippy's type_complexity warning on `insert_batch`.
