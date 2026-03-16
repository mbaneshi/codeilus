//! Integration tests for codeilus-learn: curriculum, quiz, progress.

use std::collections::HashMap;

use codeilus_core::ids::{CommunityId, FileId, SymbolId};
use codeilus_core::types::{Confidence, EdgeKind};
use codeilus_graph::{Community, GraphEdge, GraphNode, KnowledgeGraph};
use codeilus_learn::types::SectionKind;
use petgraph::graph::DiGraph;

/// Build a test graph with 3 communities, 10 symbols, and realistic edges.
fn make_test_graph() -> KnowledgeGraph {
    let mut graph = DiGraph::new();
    let mut node_index = HashMap::new();
    let mut communities = Vec::new();

    // Community 0: "Auth" — 6 symbols
    let auth_members: Vec<(i64, &str, &str)> = vec![
        (0, "login", "fn"),
        (1, "validate_token", "fn"),
        (2, "UserSession", "struct"),
        (3, "hash_password", "fn"),
        (10, "refresh_token", "fn"),
        (11, "AuthConfig", "struct"),
    ];
    let mut auth_sids = Vec::new();
    for (id, name, kind) in &auth_members {
        let sid = SymbolId(*id);
        let idx = graph.add_node(GraphNode {
            symbol_id: sid,
            file_id: FileId(0),
            name: name.to_string(),
            kind: kind.to_string(),
            community_id: Some(CommunityId(0)),
        });
        node_index.insert(sid, idx);
        auth_sids.push(sid);
    }
    communities.push(Community {
        id: CommunityId(0),
        label: "Auth".to_string(),
        members: auth_sids,
        cohesion: 0.85,
    });

    // Community 1: "Database" — 5 symbols
    let db_members: Vec<(i64, &str, &str)> = vec![
        (4, "DbPool", "struct"),
        (5, "query", "fn"),
        (6, "migrate", "fn"),
        (12, "Connection", "struct"),
        (13, "transaction", "fn"),
    ];
    let mut db_sids = Vec::new();
    for (id, name, kind) in &db_members {
        let sid = SymbolId(*id);
        let idx = graph.add_node(GraphNode {
            symbol_id: sid,
            file_id: FileId(1),
            name: name.to_string(),
            kind: kind.to_string(),
            community_id: Some(CommunityId(1)),
        });
        node_index.insert(sid, idx);
        db_sids.push(sid);
    }
    communities.push(Community {
        id: CommunityId(1),
        label: "Database".to_string(),
        members: db_sids,
        cohesion: 0.9,
    });

    // Community 2: "API" — 5 symbols
    let api_members: Vec<(i64, &str, &str)> = vec![
        (7, "handle_request", "fn"),
        (8, "Router", "struct"),
        (9, "middleware", "fn"),
        (14, "Response", "struct"),
        (15, "cors_handler", "fn"),
    ];
    let mut api_sids = Vec::new();
    for (id, name, kind) in &api_members {
        let sid = SymbolId(*id);
        let idx = graph.add_node(GraphNode {
            symbol_id: sid,
            file_id: FileId(2),
            name: name.to_string(),
            kind: kind.to_string(),
            community_id: Some(CommunityId(2)),
        });
        node_index.insert(sid, idx);
        api_sids.push(sid);
    }
    communities.push(Community {
        id: CommunityId(2),
        label: "API".to_string(),
        members: api_sids,
        cohesion: 0.75,
    });

    // Edges: Auth internals
    let n = |id: i64| *node_index.get(&SymbolId(id)).unwrap();
    let calls = |_src: i64, _tgt: i64| GraphEdge {
        kind: EdgeKind::Calls,
        confidence: Confidence::high(),
    };

    // login -> validate_token, login -> hash_password
    graph.add_edge(n(0), n(1), calls(0, 1));
    graph.add_edge(n(0), n(3), calls(0, 3));
    // validate_token -> UserSession
    graph.add_edge(n(1), n(2), calls(1, 2));

    // Database internals: query -> DbPool
    graph.add_edge(n(5), n(4), calls(5, 4));

    // API internals: handle_request -> Router, handle_request -> middleware
    graph.add_edge(n(7), n(8), calls(7, 8));
    graph.add_edge(n(7), n(9), calls(7, 9));

    // Cross-community: API -> Auth (handle_request -> login)
    graph.add_edge(n(7), n(0), calls(7, 0));
    // Auth -> Database (validate_token -> query)
    graph.add_edge(n(1), n(5), calls(1, 5));

    KnowledgeGraph {
        graph,
        node_index,
        communities,
        processes: vec![],
        entry_points: vec![codeilus_graph::EntryPoint {
            symbol_id: SymbolId(7),
            score: 0.95,
            reason: "HTTP entry point".to_string(),
        }],
    }
}

// ──────────────────────────────────────────────
// Curriculum generation tests
// ──────────────────────────────────────────────

#[test]
fn curriculum_structure_with_3_communities() {
    let kg = make_test_graph();
    let curriculum = codeilus_learn::generate_curriculum(&kg).unwrap();

    // Should have: Chapter 0 + 3 community chapters + final chapter = 5
    assert_eq!(
        curriculum.chapters.len(),
        5,
        "Expected 5 chapters (1 big picture + 3 communities + 1 final)"
    );

    assert_eq!(curriculum.chapters[0].title, "The Big Picture");
    assert_eq!(
        curriculum.chapters.last().unwrap().title,
        "Putting It All Together"
    );
}

#[test]
fn curriculum_chapter0_has_populated_content() {
    let kg = make_test_graph();
    let curriculum = codeilus_learn::generate_curriculum(&kg).unwrap();
    let ch0 = &curriculum.chapters[0];

    // Overview should mention symbol count and community count
    let overview = &ch0
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Overview)
        .unwrap()
        .content;
    assert!(
        overview.contains("16"),
        "Overview should mention 16 symbols: {overview}"
    );
    assert!(
        overview.contains("3"),
        "Overview should mention 3 modules: {overview}"
    );

    // Key Concepts should list module names
    let key_concepts = &ch0
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::KeyConcepts)
        .unwrap()
        .content;
    assert!(key_concepts.contains("Auth"), "Should list Auth module");
    assert!(
        key_concepts.contains("Database"),
        "Should list Database module"
    );
    assert!(key_concepts.contains("API"), "Should list API module");

    // CodeWalkthrough should have a reading order
    let walkthrough = &ch0
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::CodeWalkthrough)
        .unwrap()
        .content;
    assert!(
        walkthrough.contains("Reading Order"),
        "Should have reading order header"
    );
}

#[test]
fn curriculum_community_chapters_have_content() {
    let kg = make_test_graph();
    let curriculum = codeilus_learn::generate_curriculum(&kg).unwrap();

    // Check each community chapter has non-empty content
    let community_chapters: Vec<_> = curriculum
        .chapters
        .iter()
        .filter(|c| c.community_id.is_some())
        .collect();
    assert_eq!(community_chapters.len(), 3);

    for ch in &community_chapters {
        for section in &ch.sections {
            assert!(
                !section.content.is_empty(),
                "Chapter '{}' section '{}' should have content",
                ch.title,
                section.title
            );
        }
    }
}

#[test]
fn curriculum_connections_show_dependencies() {
    let kg = make_test_graph();
    let curriculum = codeilus_learn::generate_curriculum(&kg).unwrap();

    // The API chapter depends on Auth (handle_request -> login)
    let api_chapter = curriculum
        .chapters
        .iter()
        .find(|c| c.title == "API")
        .unwrap();
    let connections = &api_chapter
        .sections
        .iter()
        .find(|s| s.kind == SectionKind::Connections)
        .unwrap()
        .content;
    assert!(
        connections.contains("Auth"),
        "API connections should mention Auth dependency: {connections}"
    );
}

#[test]
fn curriculum_dependency_ordering() {
    let kg = make_test_graph();
    let curriculum = codeilus_learn::generate_curriculum(&kg).unwrap();

    // Database has no outgoing cross-community deps, so should come early
    // API depends on Auth, Auth depends on Database
    let positions: HashMap<String, usize> = curriculum
        .chapters
        .iter()
        .enumerate()
        .map(|(i, c)| (c.title.clone(), i))
        .collect();

    let db_pos = positions["Database"];
    let auth_pos = positions["Auth"];
    let api_pos = positions["API"];

    // Database should come before Auth (Auth calls Database.query)
    assert!(
        db_pos < auth_pos,
        "Database ({db_pos}) should come before Auth ({auth_pos})"
    );
    // Auth should come before API (API calls Auth.login)
    assert!(
        auth_pos < api_pos,
        "Auth ({auth_pos}) should come before API ({api_pos})"
    );
}

// ──────────────────────────────────────────────
// Quiz generation tests
// ──────────────────────────────────────────────

#[test]
fn quiz_generates_valid_questions() {
    let kg = make_test_graph();
    let curriculum = codeilus_learn::generate_curriculum(&kg).unwrap();

    for chapter in &curriculum.chapters {
        let quiz = codeilus_learn::generate_quiz(chapter, &kg).unwrap();

        assert!(
            (3..=5).contains(&quiz.questions.len()),
            "Chapter '{}' quiz should have 3-5 questions, got {}",
            chapter.title,
            quiz.questions.len()
        );

        for q in &quiz.questions {
            // correct_index must be valid
            assert!(
                q.correct_index < q.options.len(),
                "Question '{}': correct_index {} >= options len {}",
                q.question,
                q.correct_index,
                q.options.len()
            );
            // Must have non-empty explanation
            assert!(!q.explanation.is_empty(), "Question '{}' needs explanation", q.question);
            // Must have at least 2 options
            assert!(
                q.options.len() >= 2,
                "Question '{}' should have at least 2 options",
                q.question
            );
        }
    }
}

#[test]
fn quiz_has_varied_question_types() {
    let kg = make_test_graph();
    let curriculum = codeilus_learn::generate_curriculum(&kg).unwrap();

    // Community chapters with edges should produce varied question types
    let community_chapters: Vec<_> = curriculum
        .chapters
        .iter()
        .filter(|c| c.community_id.is_some())
        .collect();

    // At least one community chapter should have >= 2 different question kinds
    let has_varied = community_chapters.iter().any(|ch| {
        let quiz = codeilus_learn::generate_quiz(ch, &kg).unwrap();
        let kinds: std::collections::HashSet<_> = quiz.questions.iter().map(|q| q.kind).collect();
        kinds.len() >= 2
    });
    assert!(has_varied, "At least one community quiz should have varied question types");
}

// ──────────────────────────────────────────────
// Progress XP calculation tests
// ──────────────────────────────────────────────

#[test]
fn progress_xp_section_and_chapter_bonus() {
    use codeilus_db::{ChapterRepo, DbPool, Migrator};
    use codeilus_learn::ProgressTracker;
    use std::sync::Arc;

    let pool = DbPool::in_memory().unwrap();
    {
        let conn = pool.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().unwrap();
    }
    let db = Arc::new(pool);

    // Insert a chapter with 6 sections
    let repo = ChapterRepo::new(Arc::clone(&db));
    let ch_id = repo.insert(0, "Test Chapter", "desc", None, "beginner").unwrap();
    for kind in SectionKind::all() {
        repo.insert_section(ch_id, kind.as_str(), kind.title(), kind.as_str(), "sample content")
            .unwrap();
    }

    let tracker = ProgressTracker::new(db);

    // Complete 5 sections: 5 * 10 = 50 XP
    let section_kinds = SectionKind::all();
    for kind in &section_kinds[..5] {
        let update = tracker.complete_section(ch_id, kind.as_str()).unwrap();
        assert_eq!(update.xp_earned, 10, "Each section should award 10 XP");
    }

    // Complete last section: 10 + 50 bonus = 60 XP
    let update = tracker
        .complete_section(ch_id, section_kinds[5].as_str())
        .unwrap();
    assert_eq!(
        update.xp_earned, 60,
        "Last section should award 10 + 50 chapter bonus"
    );

    // Total: 6*10 + 50 = 110
    assert_eq!(update.total_xp, 110, "Total XP should be 110");
}

#[test]
fn progress_quiz_xp_only_on_pass() {
    use codeilus_db::{ChapterRepo, DbPool, Migrator};
    use codeilus_learn::ProgressTracker;
    use std::sync::Arc;

    let pool = DbPool::in_memory().unwrap();
    {
        let conn = pool.connection();
        let migrator = Migrator::new(&conn);
        migrator.apply_pending().unwrap();
    }
    let db = Arc::new(pool);

    let repo = ChapterRepo::new(Arc::clone(&db));
    let ch_id = repo
        .insert(0, "Quiz Chapter", "desc", None, "beginner")
        .unwrap();
    // Insert sections so progress tracking can work
    for kind in SectionKind::all() {
        repo.insert_section(ch_id, kind.as_str(), kind.title(), kind.as_str(), "sample content")
            .unwrap();
    }

    let tracker = ProgressTracker::new(db);

    // Failed quiz: 0 XP
    let fail_update = tracker.record_quiz(ch_id, 0.3, false).unwrap();
    assert_eq!(fail_update.xp_earned, 0, "Failed quiz should award 0 XP");

    // Passed quiz: 25 XP
    let pass_update = tracker.record_quiz(ch_id, 0.9, true).unwrap();
    assert_eq!(pass_update.xp_earned, 25, "Passed quiz should award 25 XP");

    // Total: 0 + 25 = 25
    assert_eq!(pass_update.total_xp, 25);
}
