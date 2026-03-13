use std::sync::{Arc, Mutex};

use codeilus_core::ids::{FileId, SymbolId};
use codeilus_db::{
    CommunityRepo, DbPool, EdgeRepo, FileMetricsRepo, FileRepo, Migrator, NarrativeRepo,
    PatternRepo, PatternRow, ProcessRepo, SymbolRepo,
};
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

// ── CommunityRepo ────────────────────────────────────────────

#[test]
fn community_repo_insert_and_list() {
    let conn = setup();
    let (s1, s2, _s3) = insert_test_symbols(&conn);
    let repo = CommunityRepo::new(conn);

    // Insert a community
    let id = repo.insert("auth_cluster", 0.85).unwrap();
    let row = repo.get(id).unwrap();
    assert_eq!(row.label, "auth_cluster");
    assert!((row.cohesion - 0.85).abs() < f64::EPSILON);

    // Add members
    repo.insert_member(id, s1).unwrap();
    repo.insert_member(id, s2).unwrap();

    let members = repo.list_members(id).unwrap();
    assert_eq!(members.len(), 2);
    assert!(members.contains(&s1));
    assert!(members.contains(&s2));

    // List all communities
    let all = repo.list().unwrap();
    assert_eq!(all.len(), 1);

    // Batch insert
    let batch_ids = repo
        .insert_batch(&[
            ("cluster_a".to_string(), 0.9),
            ("cluster_b".to_string(), 0.7),
        ])
        .unwrap();
    assert_eq!(batch_ids.len(), 2);

    let all = repo.list().unwrap();
    assert_eq!(all.len(), 3);

    // Batch members
    repo.insert_members_batch(&[(batch_ids[0], s1), (batch_ids[1], s2)])
        .unwrap();
    let members_a = repo.list_members(batch_ids[0]).unwrap();
    assert_eq!(members_a.len(), 1);

    // Delete all
    repo.delete_all().unwrap();
    let all = repo.list().unwrap();
    assert_eq!(all.len(), 0);
}

// ── ProcessRepo ──────────────────────────────────────────────

#[test]
fn pattern_repo_insert_and_list() {
    let conn = setup();
    let repo = PatternRepo::new(conn);

    let row = PatternRow {
        id: 0,
        kind: "god_class".to_string(),
        severity: "warning".to_string(),
        file_id: None,
        symbol_id: None,
        description: "BigClass has 25 methods".to_string(),
    };
    let id = repo.insert(&row).unwrap();
    assert!(id > 0);

    let all = repo.list().unwrap();
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].kind, "god_class");
    assert_eq!(all[0].severity, "warning");

    // Batch insert
    let batch = vec![
        PatternRow {
            id: 0,
            kind: "long_method".to_string(),
            severity: "error".to_string(),
            file_id: None,
            symbol_id: None,
            description: "huge_fn is 250 lines".to_string(),
        },
        PatternRow {
            id: 0,
            kind: "security_hotspot".to_string(),
            severity: "error".to_string(),
            file_id: None,
            symbol_id: None,
            description: "Hardcoded secret".to_string(),
        },
    ];
    let ids = repo.insert_batch(&batch).unwrap();
    assert_eq!(ids.len(), 2);

    let all = repo.list().unwrap();
    assert_eq!(all.len(), 3);

    // Delete all
    repo.delete_all().unwrap();
    let all = repo.list().unwrap();
    assert_eq!(all.len(), 0);
}

#[test]
fn pattern_repo_filter_by_severity() {
    let conn = setup();
    let repo = PatternRepo::new(conn);

    let batch = vec![
        PatternRow {
            id: 0,
            kind: "god_class".to_string(),
            severity: "warning".to_string(),
            file_id: None,
            symbol_id: None,
            description: "warning finding".to_string(),
        },
        PatternRow {
            id: 0,
            kind: "long_method".to_string(),
            severity: "error".to_string(),
            file_id: None,
            symbol_id: None,
            description: "error finding 1".to_string(),
        },
        PatternRow {
            id: 0,
            kind: "security_hotspot".to_string(),
            severity: "error".to_string(),
            file_id: None,
            symbol_id: None,
            description: "error finding 2".to_string(),
        },
    ];
    repo.insert_batch(&batch).unwrap();

    let errors = repo.list_by_severity("error").unwrap();
    assert_eq!(errors.len(), 2);

    let warnings = repo.list_by_severity("warning").unwrap();
    assert_eq!(warnings.len(), 1);

    let counts = repo.count_by_severity().unwrap();
    assert_eq!(counts.len(), 2);
}

// ── ProcessRepo ─────────────────────────────────────────────

#[test]
fn process_repo_insert_and_list_steps() {
    let conn = setup();
    let (s1, s2, s3) = insert_test_symbols(&conn);
    let repo = ProcessRepo::new(conn);

    // Insert process
    let proc_id = repo.insert("main_flow", s1).unwrap();
    let row = repo.get(proc_id).unwrap();
    assert_eq!(row.name, "main_flow");
    assert_eq!(row.entry_symbol_id, s1);

    // Insert steps
    repo.insert_step(proc_id, 0, s1, "entry").unwrap();
    repo.insert_step(proc_id, 1, s2, "process").unwrap();
    repo.insert_step(proc_id, 2, s3, "cleanup").unwrap();

    // List steps (should be ordered)
    let steps = repo.list_steps(proc_id).unwrap();
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0].step_order, 0);
    assert_eq!(steps[0].symbol_id, s1);
    assert_eq!(steps[1].step_order, 1);
    assert_eq!(steps[1].symbol_id, s2);
    assert_eq!(steps[2].step_order, 2);
    assert_eq!(steps[2].symbol_id, s3);

    // List all processes
    let all = repo.list().unwrap();
    assert_eq!(all.len(), 1);

    // Delete all
    repo.delete_all().unwrap();
    let all = repo.list().unwrap();
    assert_eq!(all.len(), 0);
}

// ── FileMetricsRepo ─────────────────────────────────────────

#[test]
fn file_metrics_repo_insert_and_get() {
    let conn = setup();
    let file_repo = FileRepo::new(Arc::clone(&conn));
    let file_id = file_repo.insert("metrics_test.rs", Some("rust"), 100).unwrap();

    let repo = FileMetricsRepo::new(conn);
    let id = repo.insert(file_id, 100, 5.5, 20, 3, 0.8).unwrap();
    assert!(id > 0);

    let row = repo.get_by_file(file_id).unwrap();
    assert!(row.is_some());
    let row = row.unwrap();
    assert_eq!(row.file_id, file_id);
    assert_eq!(row.sloc, 100);
    assert!((row.complexity - 5.5).abs() < f64::EPSILON);
    assert_eq!(row.churn, 20);
    assert_eq!(row.contributors, 3);
}

#[test]
fn file_metrics_repo_list_hotspots() {
    let conn = setup();
    let file_repo = FileRepo::new(Arc::clone(&conn));

    let mut file_ids = Vec::new();
    for i in 0..5 {
        let fid = file_repo
            .insert(&format!("file_{i}.rs"), Some("rust"), 100)
            .unwrap();
        file_ids.push(fid);
    }

    let repo = FileMetricsRepo::new(conn);
    for (i, fid) in file_ids.iter().enumerate() {
        let complexity = (i + 1) as f64 * 10.0;
        repo.insert(*fid, 50, complexity, 5, 2, 0.5).unwrap();
    }

    let hotspots = repo.list_hotspots(3).unwrap();
    assert_eq!(hotspots.len(), 3);
    assert!((hotspots[0].complexity - 50.0).abs() < f64::EPSILON);
    assert!((hotspots[1].complexity - 40.0).abs() < f64::EPSILON);
    assert!((hotspots[2].complexity - 30.0).abs() < f64::EPSILON);
}

// ── NarrativeRepo ─────────────────────────────────────────────

#[test]
fn narrative_repo_insert_and_get() {
    let conn = setup();
    let repo = NarrativeRepo::new(conn);

    let id = repo.insert("overview", None, "This is the overview.").unwrap();
    assert!(id > 0);

    let row = repo.get_by_kind("overview").unwrap();
    assert!(row.is_some());
    let row = row.unwrap();
    assert_eq!(row.kind, "overview");
    assert_eq!(row.content, "This is the overview.");
}

#[test]
fn narrative_repo_get_by_kind_and_target() {
    let conn = setup();
    let repo = NarrativeRepo::new(conn);

    repo.insert("module_summary", Some(1), "Summary for community 1")
        .unwrap();
    repo.insert("module_summary", Some(2), "Summary for community 2")
        .unwrap();

    let row = repo.get_by_kind_and_target("module_summary", 1).unwrap();
    assert!(row.is_some());
    assert_eq!(row.unwrap().content, "Summary for community 1");

    let row = repo.get_by_kind_and_target("module_summary", 2).unwrap();
    assert!(row.is_some());
    assert_eq!(row.unwrap().content, "Summary for community 2");
}

#[test]
fn narrative_repo_list_by_kind() {
    let conn = setup();
    let repo = NarrativeRepo::new(conn);

    repo.insert("module_summary", Some(1), "Summary 1").unwrap();
    repo.insert("module_summary", Some(2), "Summary 2").unwrap();
    repo.insert("module_summary", Some(3), "Summary 3").unwrap();
    repo.insert("overview", None, "Overview").unwrap();

    let summaries = repo.list_by_kind("module_summary").unwrap();
    assert_eq!(summaries.len(), 3);

    let overviews = repo.list_by_kind("overview").unwrap();
    assert_eq!(overviews.len(), 1);

    let all = repo.list().unwrap();
    assert_eq!(all.len(), 4);

    repo.delete_all().unwrap();
    assert_eq!(repo.list().unwrap().len(), 0);
}
