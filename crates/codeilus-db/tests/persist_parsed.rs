use std::path::PathBuf;

use codeilus_core::{Language, SymbolKind};
use codeilus_db::{DbPool, Migrator};
use codeilus_parse::{ParsedFile, Symbol};

#[test]
fn persist_parsed_inserts_files_and_symbols() {
    // In-memory database with schema applied.
    let pool = DbPool::in_memory().expect("create in-memory db");
    {
        let conn = pool.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().expect("apply migrations");
    }

    // Build a minimal parsed file with a single symbol.
    let parsed = vec![ParsedFile {
        path: PathBuf::from("src/main.rs"),
        language: Language::Rust,
        symbols: vec![Symbol {
            name: "foo".to_string(),
            kind: SymbolKind::Function,
            start_line: 1,
            end_line: 3,
            signature: Some("fn foo()".to_string()),
        }],
        imports: Vec::new(),
        calls: Vec::new(),
        heritage: Vec::new(),
    }];

    // Persist and verify that rows are created.
    pool.persist_parsed_files(&parsed)
        .expect("persist parsed files");

    let conn = pool.connection();

    let file_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM files", [], |row| row.get(0))
        .expect("count files");
    let symbol_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM symbols", [], |row| row.get(0))
        .expect("count symbols");

    assert_eq!(file_count, 1);
    assert_eq!(symbol_count, 1);
}

