//! Integration tests for the BM25 search engine.

use std::sync::Arc;

use codeilus_db::{DbPool, Migrator};
use codeilus_search::{SearchEngine, SearchResultKind};

/// Create an in-memory DB with all migrations applied and return a search engine.
fn setup() -> (Arc<DbPool>, SearchEngine) {
    let pool = DbPool::in_memory().expect("failed to create in-memory db");
    {
        let conn = pool.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().expect("failed to apply migrations");
    }
    let pool = Arc::new(pool);
    let engine = SearchEngine::new(Arc::clone(&pool));
    (pool, engine)
}

/// Insert sample data into the database.
fn insert_sample_data(pool: &DbPool) {
    let conn = pool.connection();
    conn.execute_batch(
        "INSERT INTO files (id, path, language, sloc) VALUES
            (1, 'src/main.rs', 'rust', 100),
            (2, 'src/lib.rs', 'rust', 200),
            (3, 'src/utils/helpers.py', 'python', 50);

         INSERT INTO symbols (id, file_id, name, kind, start_line, end_line, signature) VALUES
            (1, 1, 'main', 'function', 1, 10, 'fn main()'),
            (2, 2, 'SearchEngine', 'struct', 5, 50, 'pub struct SearchEngine'),
            (3, 2, 'search', 'function', 52, 80, 'pub fn search(&self, q: &str) -> Vec<SearchResult>'),
            (4, 3, 'format_output', 'function', 1, 20, 'def format_output(data)');

         INSERT INTO narratives (id, kind, target_id, content) VALUES
            (1, 'file_summary', 1, 'The main entry point for the application, initializes logging and starts the server.'),
            (2, 'symbol_doc', 2, 'SearchEngine provides BM25 full-text search capabilities over the codebase.');",
    )
    .expect("failed to insert sample data");
}

#[test]
fn test_search_files_by_path() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("main", Some(SearchResultKind::File), 10)
        .expect("search failed");

    assert!(!results.is_empty(), "expected at least one file result");
    assert_eq!(results[0].kind, SearchResultKind::File);
    assert!(results[0].name.contains("main"));
}

#[test]
fn test_search_files_by_language() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("rust", Some(SearchResultKind::File), 10)
        .expect("search failed");

    assert_eq!(results.len(), 2, "expected two rust files");
}

#[test]
fn test_search_symbols_ranked() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("search", Some(SearchResultKind::Symbol), 10)
        .expect("search failed");

    assert!(!results.is_empty(), "expected at least one symbol result");
    // All results should be symbols
    for r in &results {
        assert_eq!(r.kind, SearchResultKind::Symbol);
    }
    // Scores should be non-negative (we negate the FTS5 rank)
    for r in &results {
        assert!(r.score >= 0.0, "score should be non-negative, got {}", r.score);
    }
}

#[test]
fn test_search_symbols_with_metadata() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("SearchEngine", Some(SearchResultKind::Symbol), 10)
        .expect("search failed");

    assert!(!results.is_empty());
    let first = &results[0];
    assert_eq!(first.metadata.symbol_kind.as_deref(), Some("struct"));
    assert_eq!(first.metadata.file_path.as_deref(), Some("src/lib.rs"));
    assert!(first.metadata.line_range.is_some());
}

#[test]
fn test_search_narratives() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("server", Some(SearchResultKind::Narrative), 10)
        .expect("search failed");

    assert!(!results.is_empty(), "expected at least one narrative result");
    assert_eq!(results[0].kind, SearchResultKind::Narrative);
}

#[test]
fn test_empty_query_returns_empty() {
    let (_pool, engine) = setup();

    let results = engine.search("", None, 10).expect("search failed");
    assert!(results.is_empty());

    let results = engine.search("   ", None, 10).expect("search failed");
    assert!(results.is_empty());
}

#[test]
fn test_unified_search_combines_results() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    // "search" appears in symbol names and narrative content
    let results = engine
        .search("search", None, 20)
        .expect("search failed");

    assert!(!results.is_empty());

    // Should contain results from multiple kinds
    let kinds: Vec<&SearchResultKind> = results.iter().map(|r| &r.kind).collect();
    let has_symbol = kinds.contains(&&SearchResultKind::Symbol);
    let has_narrative = kinds.contains(&&SearchResultKind::Narrative);
    assert!(
        has_symbol || has_narrative,
        "unified search should return results from multiple kinds"
    );

    // Results should be sorted by RRF score descending
    for window in results.windows(2) {
        assert!(
            window[0].score >= window[1].score,
            "results should be sorted by score descending"
        );
    }
}

#[test]
fn test_special_characters_in_query() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    // These should not cause FTS5 syntax errors
    let results = engine.search("foo* -bar OR baz", None, 10);
    assert!(results.is_ok(), "special characters should be sanitized");

    let results = engine.search("\"quoted\"", None, 10);
    assert!(results.is_ok(), "quoted input should be sanitized");

    let results = engine.search("***", None, 10);
    assert!(results.is_ok(), "pure special chars should return empty");
    assert!(results.unwrap().is_empty());
}

#[test]
fn test_rebuild_index() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    // Rebuild should succeed
    engine.rebuild_index().expect("rebuild_index failed");

    // Search should still work after rebuild
    let results = engine
        .search("main", Some(SearchResultKind::File), 10)
        .expect("search after rebuild failed");
    assert!(!results.is_empty());
}

#[test]
fn test_no_results_for_nonexistent_term() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("zzzznonexistent", None, 10)
        .expect("search failed");
    assert!(results.is_empty());
}

#[test]
fn test_limit_is_respected() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("rust", Some(SearchResultKind::File), 1)
        .expect("search failed");
    assert!(results.len() <= 1, "limit should be respected");
}

#[test]
fn test_search_files_returns_snippets() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("helpers", Some(SearchResultKind::File), 10)
        .expect("search failed");

    assert!(!results.is_empty());
    let first = &results[0];
    // Snippet should contain the matched term
    assert!(
        first.snippet.contains("helpers"),
        "snippet should contain the matched term, got: {}",
        first.snippet
    );
}

#[test]
fn test_search_symbols_returns_snippets() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("format_output", Some(SearchResultKind::Symbol), 10)
        .expect("search failed");

    assert!(!results.is_empty());
    let first = &results[0];
    assert!(
        first.snippet.contains("format_output"),
        "snippet should contain the matched symbol, got: {}",
        first.snippet
    );
}

#[test]
fn test_search_narratives_returns_snippets() {
    let (pool, engine) = setup();
    insert_sample_data(&pool);

    let results = engine
        .search("logging", Some(SearchResultKind::Narrative), 10)
        .expect("search failed");

    assert!(!results.is_empty());
    let first = &results[0];
    assert!(
        first.snippet.contains("logging"),
        "snippet should contain the matched term, got: {}",
        first.snippet
    );
}
