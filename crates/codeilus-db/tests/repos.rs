use std::sync::{Arc, Mutex};

use codeilus_core::ids::{FileId, SymbolId};
use codeilus_db::{DbPool, EdgeRepo, FileRepo, Migrator, SymbolRepo};
use rusqlite::Connection;

fn setup() -> Arc<Mutex<Connection>> {
    let db = DbPool::in_memory().unwrap();
    let conn = db.connection();
    Migrator::new(&conn).apply_pending().unwrap();
    drop(conn);
    db.conn_arc()
}

// ── FileRepo ──────────────────────────────────────────────────

#[test]
fn file_insert_and_get() {
    let conn = setup();
    let repo = FileRepo::new(conn);
    let id = repo.insert("src/main.rs", Some("rust"), 100).unwrap();
    let row = repo.get(id).unwrap();
    assert_eq!(row.path, "src/main.rs");
    assert_eq!(row.language.as_deref(), Some("rust"));
    assert_eq!(row.sloc, 100);
}

#[test]
fn file_insert_batch() {
    let conn = setup();
    let repo = FileRepo::new(conn);
    let files = vec![
        ("a.rs".to_string(), Some("rust".to_string()), 10),
        ("b.py".to_string(), Some("python".to_string()), 20),
        ("c.go".to_string(), Some("go".to_string()), 30),
    ];
    let ids = repo.insert_batch(&files).unwrap();
    assert_eq!(ids.len(), 3);
    assert_eq!(repo.count().unwrap(), 3);
}

#[test]
fn file_get_by_path() {
    let conn = setup();
    let repo = FileRepo::new(conn);
    repo.insert("src/lib.rs", Some("rust"), 50).unwrap();
    let row = repo.get_by_path("src/lib.rs").unwrap();
    assert!(row.is_some());
    assert_eq!(row.unwrap().path, "src/lib.rs");
}

#[test]
fn file_get_by_path_not_found() {
    let conn = setup();
    let repo = FileRepo::new(conn);
    let row = repo.get_by_path("nonexistent.rs").unwrap();
    assert!(row.is_none());
}

#[test]
fn file_list_all() {
    let conn = setup();
    let repo = FileRepo::new(conn);
    let files = vec![
        ("a.rs".to_string(), Some("rust".to_string()), 10),
        ("b.py".to_string(), Some("python".to_string()), 20),
        ("c.go".to_string(), Some("go".to_string()), 30),
    ];
    repo.insert_batch(&files).unwrap();
    let all = repo.list(None).unwrap();
    assert_eq!(all.len(), 3);
}

#[test]
fn file_list_with_language_filter() {
    let conn = setup();
    let repo = FileRepo::new(conn);
    repo.insert("a.py", Some("python"), 10).unwrap();
    repo.insert("b.rs", Some("rust"), 20).unwrap();
    repo.insert("c.py", Some("python"), 30).unwrap();
    let python_files = repo.list(Some("python")).unwrap();
    assert_eq!(python_files.len(), 2);
    let rust_files = repo.list(Some("rust")).unwrap();
    assert_eq!(rust_files.len(), 1);
}

#[test]
fn file_delete_all() {
    let conn = setup();
    let repo = FileRepo::new(conn);
    let files = vec![
        ("a.rs".to_string(), Some("rust".to_string()), 10),
        ("b.rs".to_string(), Some("rust".to_string()), 20),
        ("c.rs".to_string(), Some("rust".to_string()), 30),
    ];
    repo.insert_batch(&files).unwrap();
    assert_eq!(repo.count().unwrap(), 3);
    repo.delete_all().unwrap();
    assert_eq!(repo.count().unwrap(), 0);
}

// ── SymbolRepo ────────────────────────────────────────────────

fn insert_test_file(conn: &Arc<Mutex<Connection>>) -> FileId {
    let file_repo = FileRepo::new(Arc::clone(conn));
    file_repo.insert("test.rs", Some("rust"), 100).unwrap()
}

#[test]
fn symbol_insert_and_get() {
    let conn = setup();
    let file_id = insert_test_file(&conn);
    let repo = SymbolRepo::new(conn);
    let id = repo
        .insert(file_id, "main", "Function", 1, 10, Some("fn main()"))
        .unwrap();
    let row = repo.get(id).unwrap();
    assert_eq!(row.name, "main");
    assert_eq!(row.kind, "Function");
    assert_eq!(row.start_line, 1);
    assert_eq!(row.end_line, 10);
    assert_eq!(row.signature.as_deref(), Some("fn main()"));
    assert_eq!(row.file_id, file_id);
}

#[test]
fn symbol_insert_batch() {
    let conn = setup();
    let file_id = insert_test_file(&conn);
    let repo = SymbolRepo::new(conn);
    let symbols: Vec<_> = (0..5)
        .map(|i| {
            (
                file_id,
                format!("sym_{i}"),
                "Function".to_string(),
                i as i64,
                (i + 5) as i64,
                None,
            )
        })
        .collect();
    let ids = repo.insert_batch(&symbols).unwrap();
    assert_eq!(ids.len(), 5);
    assert_eq!(repo.count().unwrap(), 5);
}

#[test]
fn symbol_list_by_file() {
    let conn = setup();
    let file_repo = FileRepo::new(Arc::clone(&conn));
    let file1 = file_repo.insert("a.rs", Some("rust"), 10).unwrap();
    let file2 = file_repo.insert("b.rs", Some("rust"), 20).unwrap();
    let repo = SymbolRepo::new(Arc::clone(&conn));
    repo.insert(file1, "alpha", "Function", 1, 5, None)
        .unwrap();
    repo.insert(file1, "beta", "Function", 6, 10, None)
        .unwrap();
    repo.insert(file2, "gamma", "Function", 1, 5, None)
        .unwrap();
    let file1_syms = repo.list_by_file(file1).unwrap();
    assert_eq!(file1_syms.len(), 2);
    let file2_syms = repo.list_by_file(file2).unwrap();
    assert_eq!(file2_syms.len(), 1);
}

#[test]
fn symbol_list_by_name() {
    let conn = setup();
    let file_id = insert_test_file(&conn);
    let repo = SymbolRepo::new(conn);
    repo.insert(file_id, "process", "Function", 1, 5, None)
        .unwrap();
    repo.insert(file_id, "handle", "Function", 6, 10, None)
        .unwrap();
    let results = repo.list_by_name("process").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].name, "process");
}

#[test]
fn symbol_search_prefix() {
    let conn = setup();
    let file_id = insert_test_file(&conn);
    let repo = SymbolRepo::new(conn);
    repo.insert(file_id, "process", "Function", 1, 5, None)
        .unwrap();
    repo.insert(file_id, "processor", "Class", 6, 20, None)
        .unwrap();
    repo.insert(file_id, "handle", "Function", 21, 30, None)
        .unwrap();
    let results = repo.search("proc").unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn symbol_delete_by_file() {
    let conn = setup();
    let file_repo = FileRepo::new(Arc::clone(&conn));
    let file1 = file_repo.insert("a.rs", Some("rust"), 10).unwrap();
    let file2 = file_repo.insert("b.rs", Some("rust"), 20).unwrap();
    let repo = SymbolRepo::new(Arc::clone(&conn));
    repo.insert(file1, "alpha", "Function", 1, 5, None)
        .unwrap();
    repo.insert(file1, "beta", "Function", 6, 10, None)
        .unwrap();
    repo.insert(file2, "gamma", "Function", 1, 5, None)
        .unwrap();
    assert_eq!(repo.count().unwrap(), 3);
    repo.delete_by_file(file1).unwrap();
    assert_eq!(repo.count().unwrap(), 1);
    let remaining = repo.list_by_file(file2).unwrap();
    assert_eq!(remaining.len(), 1);
    assert_eq!(remaining[0].name, "gamma");
}

// ── EdgeRepo ──────────────────────────────────────────────────

fn insert_test_symbols(conn: &Arc<Mutex<Connection>>) -> (SymbolId, SymbolId, SymbolId) {
    let file_repo = FileRepo::new(Arc::clone(conn));
    let file_id = file_repo.insert("test.rs", Some("rust"), 100).unwrap();
    let sym_repo = SymbolRepo::new(Arc::clone(conn));
    let s1 = sym_repo
        .insert(file_id, "caller", "Function", 1, 5, None)
        .unwrap();
    let s2 = sym_repo
        .insert(file_id, "callee", "Function", 6, 10, None)
        .unwrap();
    let s3 = sym_repo
        .insert(file_id, "module", "Module", 11, 20, None)
        .unwrap();
    (s1, s2, s3)
}

#[test]
fn edge_insert_and_list_from() {
    let conn = setup();
    let (s1, s2, _s3) = insert_test_symbols(&conn);
    let repo = EdgeRepo::new(conn);
    repo.insert(s1, s2, "CALLS", 1.0).unwrap();
    let edges = repo.list_from(s1).unwrap();
    assert_eq!(edges.len(), 1);
    assert_eq!(edges[0].source_id, s1);
    assert_eq!(edges[0].target_id, s2);
    assert_eq!(edges[0].kind, "CALLS");
}

#[test]
fn edge_insert_batch() {
    let conn = setup();
    let (s1, s2, s3) = insert_test_symbols(&conn);
    let repo = EdgeRepo::new(conn);
    let edges: Vec<_> = (0..10)
        .map(|i| {
            if i % 2 == 0 {
                (s1, s2, "CALLS".to_string(), 0.9)
            } else {
                (s2, s3, "IMPORTS".to_string(), 1.0)
            }
        })
        .collect();
    let ids = repo.insert_batch(&edges).unwrap();
    assert_eq!(ids.len(), 10);
    assert_eq!(repo.count().unwrap(), 10);
}

#[test]
fn edge_list_to() {
    let conn = setup();
    let (s1, s2, s3) = insert_test_symbols(&conn);
    let repo = EdgeRepo::new(conn);
    repo.insert(s1, s2, "CALLS", 1.0).unwrap();
    repo.insert(s3, s2, "CALLS", 0.8).unwrap();
    let incoming = repo.list_to(s2).unwrap();
    assert_eq!(incoming.len(), 2);
}

#[test]
fn edge_list_by_kind() {
    let conn = setup();
    let (s1, s2, s3) = insert_test_symbols(&conn);
    let repo = EdgeRepo::new(conn);
    repo.insert(s1, s2, "CALLS", 1.0).unwrap();
    repo.insert(s1, s3, "IMPORTS", 1.0).unwrap();
    repo.insert(s2, s3, "CALLS", 0.9).unwrap();
    let calls = repo.list_by_kind("CALLS").unwrap();
    assert_eq!(calls.len(), 2);
    let imports = repo.list_by_kind("IMPORTS").unwrap();
    assert_eq!(imports.len(), 1);
}
