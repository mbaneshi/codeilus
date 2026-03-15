use std::path::PathBuf;

use codeilus_core::{Language, SymbolKind};
use codeilus_db::{DbPool, Migrator};
use codeilus_parse::{ParsedFile, Symbol};

#[test]
fn clear_analysis_data_empties_tables() {
    let pool = DbPool::in_memory().expect("create in-memory db");
    {
        let conn = pool.connection();
        Migrator::new(&conn).apply_pending().expect("apply migrations");
    }

    // Insert some data via persist_parsed_files.
    let parsed = vec![ParsedFile {
        path: PathBuf::from("src/main.rs"),
        language: Language::Rust,
        sloc: 10,
        symbols: vec![Symbol {
            name: "main".to_string(),
            kind: SymbolKind::Function,
            start_line: 1,
            end_line: 10,
            signature: Some("fn main()".to_string()),
        }],
        imports: Vec::new(),
        calls: Vec::new(),
        heritage: Vec::new(),
    }];

    pool.persist_parsed_files(&parsed).expect("persist");

    // Verify data exists.
    {
        let conn = pool.connection();
        let file_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap();
        assert_eq!(file_count, 1, "should have 1 file before clear");
    }

    // Clear and verify empty.
    pool.clear_analysis_data().expect("clear analysis data");

    {
        let conn = pool.connection();
        let file_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
            .unwrap();
        let symbol_count: i64 = conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
            .unwrap();
        assert_eq!(file_count, 0, "files should be empty after clear");
        assert_eq!(symbol_count, 0, "symbols should be empty after clear");
    }
}

#[test]
fn re_analyze_succeeds_after_clear() {
    let pool = DbPool::in_memory().expect("create in-memory db");
    {
        let conn = pool.connection();
        Migrator::new(&conn).apply_pending().expect("apply migrations");
    }

    let parsed = vec![ParsedFile {
        path: PathBuf::from("src/lib.rs"),
        language: Language::Rust,
        sloc: 5,
        symbols: vec![Symbol {
            name: "helper".to_string(),
            kind: SymbolKind::Function,
            start_line: 1,
            end_line: 5,
            signature: None,
        }],
        imports: Vec::new(),
        calls: Vec::new(),
        heritage: Vec::new(),
    }];

    // First analyze.
    pool.persist_parsed_files(&parsed).expect("first persist");

    // Clear + re-analyze (simulating what run_analyze now does).
    pool.clear_analysis_data().expect("clear");
    pool.persist_parsed_files(&parsed).expect("second persist should not fail");

    let conn = pool.connection();
    let file_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
        .unwrap();
    assert_eq!(file_count, 1, "should have exactly 1 file after re-analyze");
}
