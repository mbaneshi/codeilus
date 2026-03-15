use std::collections::HashMap;
use std::path::PathBuf;

use codeilus_core::ids::{CommunityId, FileId, SymbolId};
use codeilus_core::types::{Confidence, EdgeKind, NarrativeKind};
use codeilus_core::Language;
use codeilus_graph::{Community, EntryPoint, GraphEdge, GraphNode, KnowledgeGraph};
use codeilus_narrate::{prompts, NarrativeGenerator};
use codeilus_parse::{ParsedFile, Symbol};
use petgraph::graph::DiGraph;

fn empty_graph() -> KnowledgeGraph {
    KnowledgeGraph {
        graph: DiGraph::new(),
        node_index: HashMap::new(),
        communities: Vec::new(),
        processes: Vec::new(),
        entry_points: Vec::new(),
    }
}

fn graph_with_communities() -> KnowledgeGraph {
    let mut graph = DiGraph::new();
    let n1 = graph.add_node(GraphNode {
        symbol_id: SymbolId(1),
        file_id: FileId(1),
        name: "main".to_string(),
        kind: "Function".to_string(),
        community_id: Some(CommunityId(1)),
    });
    let n2 = graph.add_node(GraphNode {
        symbol_id: SymbolId(2),
        file_id: FileId(1),
        name: "helper".to_string(),
        kind: "Function".to_string(),
        community_id: Some(CommunityId(1)),
    });
    let n3 = graph.add_node(GraphNode {
        symbol_id: SymbolId(3),
        file_id: FileId(2),
        name: "DbPool".to_string(),
        kind: "Struct".to_string(),
        community_id: Some(CommunityId(2)),
    });
    graph.add_edge(
        n1,
        n2,
        GraphEdge {
            kind: EdgeKind::Calls,
            confidence: Confidence::certain(),
        },
    );
    graph.add_edge(
        n1,
        n3,
        GraphEdge {
            kind: EdgeKind::Imports,
            confidence: Confidence::certain(),
        },
    );

    let node_index = HashMap::from([
        (SymbolId(1), n1),
        (SymbolId(2), n2),
        (SymbolId(3), n3),
    ]);

    KnowledgeGraph {
        graph,
        node_index,
        communities: vec![
            Community {
                id: CommunityId(1),
                label: "core_logic".to_string(),
                members: vec![SymbolId(1), SymbolId(2)],
                cohesion: 0.9,
            },
            Community {
                id: CommunityId(2),
                label: "database".to_string(),
                members: vec![SymbolId(3)],
                cohesion: 0.7,
            },
        ],
        processes: Vec::new(),
        entry_points: vec![
            EntryPoint {
                symbol_id: SymbolId(1),
                score: 0.95,
                reason: "Main entry point".to_string(),
            },
            EntryPoint {
                symbol_id: SymbolId(3),
                score: 0.6,
                reason: "Database setup".to_string(),
            },
        ],
    }
}

fn sample_parsed_files() -> Vec<ParsedFile> {
    vec![
        ParsedFile {
            path: PathBuf::from("src/main.rs"),
            language: Language::Rust,
            sloc: 100,
            symbols: vec![Symbol {
                name: "main".to_string(),
                kind: codeilus_core::SymbolKind::Function,
                start_line: 1,
                end_line: 50,
                signature: Some("fn main()".to_string()),
            }],
            imports: Vec::new(),
            calls: Vec::new(),
            heritage: Vec::new(),
        },
        ParsedFile {
            path: PathBuf::from("src/db.py"),
            language: Language::Python,
            sloc: 200,
            symbols: vec![Symbol {
                name: "DbPool".to_string(),
                kind: codeilus_core::SymbolKind::Class,
                start_line: 1,
                end_line: 100,
                signature: None,
            }],
            imports: Vec::new(),
            calls: Vec::new(),
            heritage: Vec::new(),
        },
    ]
}

// ── Prompt Template Tests ─────────────────────────────────────

#[test]
fn prompt_template_overview() {
    let prompt = prompts::get_prompt(NarrativeKind::Overview);
    assert!(!prompt.system.is_empty());
    assert!(!prompt.user_template.is_empty());
}

#[test]
fn prompt_template_all_kinds() {
    let kinds = [
        NarrativeKind::Overview,
        NarrativeKind::Architecture,
        NarrativeKind::ReadingOrder,
        NarrativeKind::ExtensionGuide,
        NarrativeKind::ContributionGuide,
        NarrativeKind::WhyTrending,
        NarrativeKind::ModuleSummary,
        NarrativeKind::SymbolExplanation,
    ];
    for kind in &kinds {
        let prompt = prompts::get_prompt(*kind);
        assert!(!prompt.system.is_empty(), "empty system for {:?}", kind);
        assert!(
            !prompt.user_template.is_empty(),
            "empty template for {:?}",
            kind
        );
    }
}

#[test]
fn prompt_template_has_context_placeholder() {
    let kinds = [
        NarrativeKind::Overview,
        NarrativeKind::Architecture,
        NarrativeKind::ReadingOrder,
        NarrativeKind::ExtensionGuide,
        NarrativeKind::ContributionGuide,
        NarrativeKind::WhyTrending,
        NarrativeKind::ModuleSummary,
        NarrativeKind::SymbolExplanation,
    ];
    for kind in &kinds {
        let prompt = prompts::get_prompt(*kind);
        assert!(
            prompt.user_template.contains("{context}"),
            "template for {:?} missing {{context}} placeholder",
            kind
        );
    }
}

// ── Placeholder Tests ─────────────────────────────────────────

#[test]
fn placeholder_overview() {
    use codeilus_narrate::placeholders;
    let graph = empty_graph();
    let files = sample_parsed_files();
    let content = placeholders::placeholder_for(NarrativeKind::Overview, &graph, &files, None);
    assert!(content.contains("2 files"), "should mention file count: {}", content);
}

#[test]
fn placeholder_reading_order() {
    use codeilus_narrate::placeholders;
    let graph = graph_with_communities();
    let content =
        placeholders::placeholder_for(NarrativeKind::ReadingOrder, &graph, &[], None);
    assert!(content.contains("main"), "should list entry point: {}", content);
    assert!(content.contains("score"), "should show scores: {}", content);
}

#[test]
fn placeholder_module_summary() {
    use codeilus_narrate::placeholders;
    let graph = graph_with_communities();
    let content = placeholders::placeholder_for(
        NarrativeKind::ModuleSummary,
        &graph,
        &[],
        Some(1),
    );
    assert!(
        content.contains("core_logic"),
        "should mention community label: {}",
        content
    );
    assert!(
        content.contains("main"),
        "should list member names: {}",
        content
    );
}

// ── Generator Tests ───────────────────────────────────────────

#[tokio::test]
async fn generator_placeholder_mode() {
    let gen = NarrativeGenerator::placeholder_only();
    let graph = graph_with_communities();
    let files = sample_parsed_files();
    let narratives = gen
        .generate_all(&graph, &files, std::path::Path::new("/tmp"))
        .await
        .unwrap();

    for n in &narratives {
        assert!(n.is_placeholder, "should be placeholder: {:?}", n.kind);
    }
}

#[tokio::test]
async fn generator_placeholder_mode_generates_all_kinds() {
    let gen = NarrativeGenerator::placeholder_only();
    let graph = graph_with_communities();
    let files = sample_parsed_files();
    let narratives = gen
        .generate_all(&graph, &files, std::path::Path::new("/tmp"))
        .await
        .unwrap();

    // 6 global + 2 communities = 8
    assert_eq!(narratives.len(), 8, "expected 8 narratives, got {}", narratives.len());

    // Check all global kinds are present
    assert!(narratives.iter().any(|n| n.kind == NarrativeKind::Overview));
    assert!(narratives.iter().any(|n| n.kind == NarrativeKind::Architecture));
    assert!(narratives.iter().any(|n| n.kind == NarrativeKind::ReadingOrder));
    assert!(narratives.iter().any(|n| n.kind == NarrativeKind::ExtensionGuide));
    assert!(narratives.iter().any(|n| n.kind == NarrativeKind::ContributionGuide));
    assert!(narratives.iter().any(|n| n.kind == NarrativeKind::WhyTrending));

    // Per-community module summaries
    let summaries: Vec<_> = narratives
        .iter()
        .filter(|n| n.kind == NarrativeKind::ModuleSummary)
        .collect();
    assert_eq!(summaries.len(), 2, "should have 2 module summaries");

    // All placeholders should be marked as such
    for n in &narratives {
        assert!(n.is_placeholder, "placeholder mode should mark all as placeholder: {:?}", n.kind);
    }
}

#[tokio::test]
async fn generator_all_kinds_covered() {
    let gen = NarrativeGenerator::placeholder_only();
    let graph = graph_with_communities();
    let files = sample_parsed_files();
    let narratives = gen
        .generate_all(&graph, &files, std::path::Path::new("/tmp"))
        .await
        .unwrap();

    // 6 global + 2 communities = 8
    assert!(
        narratives.len() >= 8,
        "expected at least 8 narratives, got {}",
        narratives.len()
    );

    // Check all global kinds are present
    assert!(narratives.iter().any(|n| n.kind == NarrativeKind::Overview));
    assert!(narratives
        .iter()
        .any(|n| n.kind == NarrativeKind::Architecture));
    assert!(narratives
        .iter()
        .any(|n| n.kind == NarrativeKind::ReadingOrder));
    assert!(narratives
        .iter()
        .any(|n| n.kind == NarrativeKind::ExtensionGuide));
    assert!(narratives
        .iter()
        .any(|n| n.kind == NarrativeKind::ContributionGuide));
    assert!(narratives
        .iter()
        .any(|n| n.kind == NarrativeKind::WhyTrending));

    // Per-community module summaries
    let summaries: Vec<_> = narratives
        .iter()
        .filter(|n| n.kind == NarrativeKind::ModuleSummary)
        .collect();
    assert_eq!(summaries.len(), 2, "should have 2 module summaries");
}

#[tokio::test]
async fn narrative_content_not_empty() {
    let gen = NarrativeGenerator::placeholder_only();
    let graph = graph_with_communities();
    let files = sample_parsed_files();
    let narratives = gen
        .generate_all(&graph, &files, std::path::Path::new("/tmp"))
        .await
        .unwrap();

    for n in &narratives {
        assert!(
            !n.content.is_empty(),
            "narrative {:?} has empty content",
            n.kind
        );
        assert!(
            !n.title.is_empty(),
            "narrative {:?} has empty title",
            n.kind
        );
    }
}

#[tokio::test]
async fn skip_llm_env_var_uses_placeholders() {
    // Set the env var for this test
    std::env::set_var("CODEILUS_SKIP_LLM", "1");
    let gen = NarrativeGenerator::new().await;
    let graph = graph_with_communities();
    let files = sample_parsed_files();
    let narratives = gen
        .generate_all(&graph, &files, std::path::Path::new("/tmp"))
        .await
        .unwrap();

    // All should be placeholders since LLM is skipped
    for n in &narratives {
        assert!(
            n.is_placeholder,
            "CODEILUS_SKIP_LLM=1 should produce placeholders: {:?}",
            n.kind
        );
    }

    // Should still produce all narrative kinds
    assert!(narratives.len() >= 8, "should have at least 8 narratives");
    assert!(narratives.iter().any(|n| n.kind == NarrativeKind::Overview));
    assert!(narratives.iter().any(|n| n.kind == NarrativeKind::ModuleSummary));

    // Clean up
    std::env::remove_var("CODEILUS_SKIP_LLM");
}
