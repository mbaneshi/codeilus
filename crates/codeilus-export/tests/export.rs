use codeilus_db::{DbPool, Migrator};
use codeilus_export::*;

/// Create an in-memory DB with migrations applied and some test data inserted.
fn setup_db() -> DbPool {
    let db = DbPool::in_memory().unwrap();
    Migrator::new(&db.connection()).apply_pending().unwrap();
    db
}

/// Insert test data into the DB: files, symbols, narratives, communities, patterns, metrics.
fn seed_db(db: &DbPool) {
    let conn = db.conn_arc();

    // Files
    let file_repo = codeilus_db::FileRepo::new(conn.clone());
    let fids = file_repo
        .insert_batch(&[
            ("src/main.rs".to_string(), Some("rust".to_string()), 100),
            ("src/lib.rs".to_string(), Some("rust".to_string()), 200),
            ("src/utils.py".to_string(), Some("python".to_string()), 50),
        ])
        .unwrap();

    // Symbols
    let sym_repo = codeilus_db::SymbolRepo::new(conn.clone());
    sym_repo
        .insert_batch(&[
            (fids[0], "main".to_string(), "function".to_string(), 1, 10, None),
            (fids[1], "Config".to_string(), "struct".to_string(), 1, 20, None),
            (fids[1], "run".to_string(), "function".to_string(), 22, 50, None),
            (fids[2], "helper".to_string(), "function".to_string(), 1, 15, None),
        ])
        .unwrap();

    // Narratives
    let narr_repo = codeilus_db::NarrativeRepo::new(conn.clone());
    narr_repo.insert("overview", None, "This is a test project for export.", false).unwrap();
    narr_repo.insert("architecture", None, "The architecture has two modules.", false).unwrap();
    narr_repo
        .insert(
            "reading_order",
            None,
            "1. src/main.rs — The entry point\n2. src/lib.rs — Core library",
            false,
        )
        .unwrap();
    narr_repo.insert("extension_guide", None, "Add new modules in src/.", false).unwrap();
    narr_repo.insert("contribution_guide", None, "Fork and submit PRs.", false).unwrap();
    narr_repo.insert("why_trending", None, "Great developer experience.", false).unwrap();

    // Communities
    let comm_repo = codeilus_db::CommunityRepo::new(conn.clone());
    let cids = comm_repo
        .insert_batch(&[("core".to_string(), 0.85), ("utils".to_string(), 0.72)])
        .unwrap();

    // Module summaries for communities
    narr_repo
        .insert("module_summary", Some(cids[0].0), "Core module handles main logic.", false)
        .unwrap();
    narr_repo
        .insert("module_summary", Some(cids[1].0), "Utils provides helper functions.", false)
        .unwrap();

    // Patterns
    let pat_repo = codeilus_db::PatternRepo::new(conn.clone());
    pat_repo
        .insert(&codeilus_db::PatternRow {
            id: 0,
            kind: "long_method".to_string(),
            severity: "warning".to_string(),
            file_id: Some(fids[1].0),
            symbol_id: None,
            description: "Method 'run' is too long (28 lines)".to_string(),
        })
        .unwrap();

    // File metrics
    let metrics_repo = codeilus_db::FileMetricsRepo::new(conn.clone());
    metrics_repo.insert(fids[0], 100, 5.0, 10, 2, 0.8).unwrap();
    metrics_repo.insert(fids[1], 200, 12.0, 25, 3, 0.9).unwrap();
}

fn make_export_data(db: &DbPool) -> ExportData {
    codeilus_export::data_loader::load_export_data("test-repo", db).unwrap()
}

// Test 1: load_export_data produces ExportData with non-empty fields
#[test]
fn export_data_all_fields() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);

    assert_eq!(data.repo_name, "test-repo");
    assert!(!data.overview.is_empty());
    assert!(!data.architecture_narrative.is_empty());
    assert!(!data.extension_guide.is_empty());
    assert!(!data.contribution_guide.is_empty());
    assert!(!data.why_trending.is_empty());
    assert!(!data.language_badges.is_empty());
    assert!(!data.communities.is_empty());
    assert!(!data.patterns.is_empty());
    assert!(data.metrics_snapshot.total_files > 0);
    assert!(data.metrics_snapshot.total_symbols > 0);
}

// Test 2: Empty DB → ExportData with empty strings (not error)
#[test]
fn export_data_empty_db() {
    let db = setup_db();
    let data = make_export_data(&db);

    assert_eq!(data.repo_name, "test-repo");
    assert!(data.overview.is_empty());
    assert!(data.language_badges.is_empty());
    assert!(data.communities.is_empty());
    assert!(data.patterns.is_empty());
    assert_eq!(data.metrics_snapshot.total_files, 0);
}

// Test 3: render_html creates a valid HTML file at the specified path
#[test]
fn render_html_creates_file() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);

    let tmpdir = std::env::temp_dir().join("codeilus-export-test-create");
    let _ = std::fs::remove_dir_all(&tmpdir);
    let output_path = tmpdir.join("test-repo.html");
    codeilus_export::renderer::render_html(&data, &output_path).unwrap();
    assert!(output_path.exists());
    let content = std::fs::read_to_string(&output_path).unwrap();
    assert!(content.starts_with("<!DOCTYPE html>"));
    let _ = std::fs::remove_dir_all(&tmpdir);
}

// Test 4: Generated HTML is <500KB
#[test]
fn render_html_size_limit() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);
    let html = codeilus_export::renderer::render_html_string(&data).unwrap();
    let size_kb = html.len() / 1024;
    assert!(
        size_kb < 500,
        "HTML size is {}KB, exceeds 500KB limit",
        size_kb
    );
}

// Test 5: HTML contains <script id="codeilus-data"> with valid JSON
#[test]
fn render_html_contains_data() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);
    let html = codeilus_export::renderer::render_html_string(&data).unwrap();
    assert!(html.contains(r#"<script id="codeilus-data" type="application/json">"#));

    // Extract JSON and validate it
    let start = html.find(r#"type="application/json">"#).unwrap() + r#"type="application/json">"#.len();
    let end = html[start..].find("</script>").unwrap() + start;
    let json_str = &html[start..end];
    let parsed: serde_json::Value = serde_json::from_str(json_str).unwrap();
    assert!(parsed.is_object());
    assert_eq!(parsed["repo_name"], "test-repo");
}

// Test 6: HTML has no external resource references that block rendering (except mermaid CDN fallback)
#[test]
fn render_html_self_contained() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);
    let html = codeilus_export::renderer::render_html_string(&data).unwrap();

    // Should not have <link rel="stylesheet" href="..."> external refs
    assert!(!html.contains("<link rel=\"stylesheet\""));
    // The only external script is the mermaid CDN fallback, which is loaded dynamically
    // No blocking <script src="..."> tags
    let blocking_script_count = html.matches("<script src=").count();
    assert_eq!(blocking_script_count, 0, "No blocking external scripts");
}

// Test 7: HTML has all 10 sections
#[test]
fn render_html_valid_structure() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);
    let html = codeilus_export::renderer::render_html_string(&data).unwrap();

    let sections = [
        "id=\"overview\"",
        "id=\"architecture\"",
        "id=\"key-files\"",
        "id=\"entry-points\"",
        "id=\"how-it-works\"",
        "id=\"how-to-extend\"",
        "id=\"how-to-contribute\"",
        "id=\"why-trending\"",
        "id=\"metrics\"",
        "id=\"deep-dive\"",
    ];
    for section in &sections {
        assert!(
            html.contains(section),
            "Missing section: {}",
            section
        );
    }
}

// Test 8: 3 languages → 3 badge entries with percentages summing to ~100
#[test]
fn language_badges() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);

    // We inserted 2 rust files and 1 python file
    assert_eq!(data.language_badges.len(), 2);
    let total: f64 = data.language_badges.iter().map(|b| b.percentage).sum();
    assert!(
        (total - 100.0).abs() < 1.0,
        "Badge percentages sum to {}, expected ~100",
        total
    );
}

// Test 9: Generate index with 3 repos → HTML contains all 3 repo names
#[test]
fn index_page_lists_repos() {
    let repos = vec![
        ExportedRepo {
            name: "alpha".to_string(),
            description: Some("First repo".to_string()),
            language: Some("Rust".to_string()),
            file_path: "alpha.html".to_string(),
            file_size_kb: 42,
            exported_at: "2026-03-13".to_string(),
        },
        ExportedRepo {
            name: "beta".to_string(),
            description: None,
            language: Some("Python".to_string()),
            file_path: "beta.html".to_string(),
            file_size_kb: 38,
            exported_at: "2026-03-13".to_string(),
        },
        ExportedRepo {
            name: "gamma".to_string(),
            description: Some("Third repo".to_string()),
            language: None,
            file_path: "gamma.html".to_string(),
            file_size_kb: 55,
            exported_at: "2026-03-13".to_string(),
        },
    ];

    let tmpdir = std::env::temp_dir().join("codeilus-export-test-index");
    let _ = std::fs::remove_dir_all(&tmpdir);
    let path = codeilus_export::index::generate_index(&repos, "2026-03-13", &tmpdir).unwrap();
    let html = std::fs::read_to_string(&path).unwrap();

    assert!(html.contains("alpha"));
    assert!(html.contains("beta"));
    assert!(html.contains("gamma"));
    assert!(html.contains("First repo"));
    assert!(html.contains("Third repo"));
    let _ = std::fs::remove_dir_all(&tmpdir);
}

// Test 10: Index page is <50KB
#[test]
fn index_page_size() {
    let repos = vec![ExportedRepo {
        name: "test".to_string(),
        description: None,
        language: None,
        file_path: "test.html".to_string(),
        file_size_kb: 10,
        exported_at: "2026-03-13".to_string(),
    }];

    let tmpdir = std::env::temp_dir().join("codeilus-export-test-index-size");
    let _ = std::fs::remove_dir_all(&tmpdir);
    let path = codeilus_export::index::generate_index(&repos, "2026-03-13", &tmpdir).unwrap();
    let size = std::fs::metadata(&path).unwrap().len();
    assert!(
        size < 50 * 1024,
        "Index page is {} bytes, exceeds 50KB limit",
        size
    );
    let _ = std::fs::remove_dir_all(&tmpdir);
}

// Test 11: "owner/repo" → "owner-repo.html"
#[test]
fn export_filename_sanitized() {
    let db = setup_db();
    seed_db(&db);
    let tmpdir = std::env::temp_dir().join("codeilus-export-test-sanitize");
    let _ = std::fs::remove_dir_all(&tmpdir);
    let path = codeilus_export::export_repo("owner/repo", &db, &tmpdir).unwrap();
    assert_eq!(path.file_name().unwrap().to_str().unwrap(), "owner-repo.html");
    let _ = std::fs::remove_dir_all(&tmpdir);
}

// Test 12: Output HTML contains Mermaid initialization code
#[test]
fn mermaid_inlined() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);
    let html = codeilus_export::renderer::render_html_string(&data).unwrap();
    assert!(html.contains("mermaid"));
    assert!(html.contains("mermaid.initialize"));
}

// Test 13: Footer contains "Generated by Codeilus" and a timestamp
#[test]
fn render_html_has_footer() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);
    let html = codeilus_export::renderer::render_html_string(&data).unwrap();
    assert!(html.contains("Generated by"), "Missing footer");
    assert!(html.contains("Codeilus"), "Footer missing Codeilus brand");
    // Timestamp should contain UTC
    assert!(html.contains("UTC"), "Footer missing timestamp");
}

// Test 14: Reading order is parsed correctly from narrative text
#[test]
fn reading_order_parsed() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);
    assert_eq!(data.reading_order.len(), 2);
    assert_eq!(data.reading_order[0].path, "src/main.rs");
    assert_eq!(data.reading_order[0].reason, "The entry point");
    assert_eq!(data.reading_order[1].path, "src/lib.rs");
}

// Test 15: File tree is generated from file paths
#[test]
fn file_tree_generated() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);
    assert!(!data.file_tree.is_empty(), "File tree should not be empty");
    assert!(data.file_tree.contains("src/main.rs"));
    assert!(data.file_tree.contains("src/lib.rs"));
}

// Test 16: Communities have module summaries
#[test]
fn communities_with_summaries() {
    let db = setup_db();
    seed_db(&db);
    let data = make_export_data(&db);
    assert_eq!(data.communities.len(), 2);
    let core = data.communities.iter().find(|c| c.label == "core").unwrap();
    assert!(core.summary.contains("Core module"));
    let utils = data.communities.iter().find(|c| c.label == "utils").unwrap();
    assert!(utils.summary.contains("Utils provides"));
}
