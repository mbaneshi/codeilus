use std::collections::HashMap;
use std::path::PathBuf;

use codeilus_core::ids::{CommunityId, FileId, SymbolId};
use codeilus_core::types::{Confidence, EdgeKind, Language, SymbolKind};
use codeilus_graph::types::{Community, GraphEdge, GraphNode};
use codeilus_graph::KnowledgeGraph;
use codeilus_metrics::types::FileMetrics;
use codeilus_parse::{ParsedFile, Symbol};
use petgraph::graph::DiGraph;

// --- Helpers ---

fn make_parsed_file(
    path: &str,
    lang: Language,
    symbols: Vec<Symbol>,
) -> ParsedFile {
    ParsedFile {
        path: PathBuf::from(path),
        language: lang,
        sloc: 10,
        symbols,
        imports: vec![],
        calls: vec![],
        heritage: vec![],
    }
}

fn make_symbol(name: &str, kind: SymbolKind, start: i64, end: i64) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind,
        start_line: start,
        end_line: end,
        signature: Some(format!("fn {name}()")),
    }
}

fn make_simple_graph() -> KnowledgeGraph {
    let mut graph = DiGraph::new();
    let mut node_index = HashMap::new();

    let a = graph.add_node(GraphNode {
        symbol_id: SymbolId(1),
        file_id: FileId(1),
        name: "func_a".to_string(),
        kind: "Function".to_string(),
        community_id: Some(CommunityId(1)),
    });
    let b = graph.add_node(GraphNode {
        symbol_id: SymbolId(2),
        file_id: FileId(1),
        name: "func_b".to_string(),
        kind: "Function".to_string(),
        community_id: Some(CommunityId(1)),
    });
    let c = graph.add_node(GraphNode {
        symbol_id: SymbolId(3),
        file_id: FileId(2),
        name: "func_c".to_string(),
        kind: "Function".to_string(),
        community_id: Some(CommunityId(2)),
    });

    node_index.insert(SymbolId(1), a);
    node_index.insert(SymbolId(2), b);
    node_index.insert(SymbolId(3), c);

    // A→B, C→B
    graph.add_edge(a, b, GraphEdge {
        kind: EdgeKind::Calls,
        confidence: Confidence::certain(),
    });
    graph.add_edge(c, b, GraphEdge {
        kind: EdgeKind::Calls,
        confidence: Confidence::certain(),
    });

    let communities = vec![
        Community {
            id: CommunityId(1),
            label: "cluster_a".to_string(),
            members: vec![SymbolId(1), SymbolId(2)],
            cohesion: 0.8,
        },
        Community {
            id: CommunityId(2),
            label: "cluster_c".to_string(),
            members: vec![SymbolId(3)],
            cohesion: 1.0,
        },
    ];

    KnowledgeGraph {
        graph,
        node_index,
        communities,
        processes: vec![],
        entry_points: vec![],
    }
}

// --- SLOC Tests ---

#[test]
fn sloc_python() {
    let source = r#"# This is a comment
import os

def main():
    # another comment
    x = 1
    return x
"#;
    let count = codeilus_metrics::sloc::count_sloc(source, Language::Python);
    // Lines: import os, def main():, x = 1, return x = 4 non-blank non-comment
    assert_eq!(count, 4, "Expected 4 SLOC for Python, got {count}");
}

#[test]
fn sloc_rust() {
    let source = r#"// This is a comment
/* Block comment
   spanning lines */
use std::fs;

fn main() {
    let x = 1;
}
"#;
    let count = codeilus_metrics::sloc::count_sloc(source, Language::Rust);
    // Lines: use std::fs;, fn main() {, let x = 1;, } = 4
    assert_eq!(count, 4, "Expected 4 SLOC for Rust, got {count}");
}

// --- Fan Tests ---

#[test]
fn fan_in_out() {
    let kg = make_simple_graph();
    let fan = codeilus_metrics::fan::compute_fan(&kg);

    // B has fan_in=2 (from A and C), fan_out=0
    let (b_in, b_out) = fan[&SymbolId(2)];
    assert_eq!(b_in, 2, "B fan_in should be 2");
    assert_eq!(b_out, 0, "B fan_out should be 0");

    // A has fan_out=1
    let (a_in, a_out) = fan[&SymbolId(1)];
    assert_eq!(a_out, 1, "A fan_out should be 1");
    assert_eq!(a_in, 0, "A fan_in should be 0");
}

// --- Complexity Tests ---

#[test]
fn complexity_simple_function() {
    let lines = vec![
        "fn process(x: i32) -> i32 {",
        "    if x > 0 {",
        "        if x > 10 {",
        "            return x * 2;",
        "        }",
        "        return x;",
        "    }",
        "    0",
        "}",
    ];
    let c = codeilus_metrics::complexity::estimate_complexity(&lines);
    // Base 1 + 2 ifs = 3
    assert!(
        (c - 3.0).abs() < 0.01,
        "Expected complexity ~3, got {c}"
    );
}

#[test]
fn complexity_fallback() {
    let c = codeilus_metrics::complexity::estimate_from_loc(50);
    // 1 + 50/10 = 6.0
    assert!(
        (c - 6.0).abs() < 0.01,
        "Expected complexity 6.0 from LOC=50, got {c}"
    );
}

// --- Modularity Tests ---

#[test]
fn modularity_q_score() {
    // Build two perfect clusters with internal edges only
    let mut graph = DiGraph::new();
    let mut node_index = HashMap::new();

    // Cluster 1: a0, a1, a2
    let a0 = graph.add_node(GraphNode {
        symbol_id: SymbolId(1), file_id: FileId(1),
        name: "a0".to_string(), kind: "Function".to_string(),
        community_id: Some(CommunityId(1)),
    });
    let a1 = graph.add_node(GraphNode {
        symbol_id: SymbolId(2), file_id: FileId(1),
        name: "a1".to_string(), kind: "Function".to_string(),
        community_id: Some(CommunityId(1)),
    });
    let a2 = graph.add_node(GraphNode {
        symbol_id: SymbolId(3), file_id: FileId(1),
        name: "a2".to_string(), kind: "Function".to_string(),
        community_id: Some(CommunityId(1)),
    });

    // Cluster 2: b0, b1, b2
    let b0 = graph.add_node(GraphNode {
        symbol_id: SymbolId(4), file_id: FileId(2),
        name: "b0".to_string(), kind: "Function".to_string(),
        community_id: Some(CommunityId(2)),
    });
    let b1 = graph.add_node(GraphNode {
        symbol_id: SymbolId(5), file_id: FileId(2),
        name: "b1".to_string(), kind: "Function".to_string(),
        community_id: Some(CommunityId(2)),
    });
    let b2 = graph.add_node(GraphNode {
        symbol_id: SymbolId(6), file_id: FileId(2),
        name: "b2".to_string(), kind: "Function".to_string(),
        community_id: Some(CommunityId(2)),
    });

    node_index.insert(SymbolId(1), a0);
    node_index.insert(SymbolId(2), a1);
    node_index.insert(SymbolId(3), a2);
    node_index.insert(SymbolId(4), b0);
    node_index.insert(SymbolId(5), b1);
    node_index.insert(SymbolId(6), b2);

    let edge = GraphEdge { kind: EdgeKind::Calls, confidence: Confidence::certain() };
    graph.add_edge(a0, a1, edge.clone());
    graph.add_edge(a1, a2, edge.clone());
    graph.add_edge(a0, a2, edge.clone());
    graph.add_edge(b0, b1, edge.clone());
    graph.add_edge(b1, b2, edge.clone());
    graph.add_edge(b0, b2, edge.clone());

    let kg = KnowledgeGraph {
        graph,
        node_index,
        communities: vec![
            Community {
                id: CommunityId(1), label: "c1".to_string(),
                members: vec![SymbolId(1), SymbolId(2), SymbolId(3)],
                cohesion: 1.0,
            },
            Community {
                id: CommunityId(2), label: "c2".to_string(),
                members: vec![SymbolId(4), SymbolId(5), SymbolId(6)],
                cohesion: 1.0,
            },
        ],
        processes: vec![],
        entry_points: vec![],
    };

    let (q, _per_community) = codeilus_metrics::modularity::compute_modularity(&kg);
    // Two perfect clusters → Q should be positive, close to 0.5
    assert!(q > 0.3, "Expected Q > 0.3 for two perfect clusters, got {q}");
}

// --- TF-IDF Tests ---

#[test]
fn tfidf_keywords() {
    let community_names = vec![
        vec!["parseFile".to_string(), "parseToken".to_string(), "readStream".to_string()],
        vec!["writeLog".to_string(), "formatOutput".to_string()],
    ];

    let results = codeilus_metrics::tfidf::compute_tfidf(&community_names, 10);
    assert_eq!(results.len(), 2);

    // "parse" should be the top keyword in community 0
    let top = &results[0];
    assert!(!top.is_empty());
    assert_eq!(top[0].0, "parse", "Expected 'parse' as top keyword, got '{}'", top[0].0);
}

#[test]
fn tfidf_camel_case_split() {
    let tokens = codeilus_metrics::tfidf::tokenize("parseJSONFile");
    assert_eq!(tokens, vec!["parse", "json", "file"]);
}

// --- Git Tests ---

#[test]
fn git_metrics_non_repo() {
    let dir = tempfile::tempdir().unwrap();
    let result = codeilus_metrics::git::compute_git_metrics(dir.path(), 100);
    assert!(result.is_empty(), "Non-git dir should return empty metrics");
}

// --- Heatmap Tests ---

#[test]
fn heatmap_scoring() {
    let mut metrics = vec![
        FileMetrics {
            file_id: FileId(1), path: "simple.py".to_string(),
            sloc: 10, complexity: 1.0, churn: 1, contributors: 1, heatmap_score: 0.0,
        },
        FileMetrics {
            file_id: FileId(2), path: "complex.py".to_string(),
            sloc: 500, complexity: 50.0, churn: 100, contributors: 10, heatmap_score: 0.0,
        },
        FileMetrics {
            file_id: FileId(3), path: "medium.py".to_string(),
            sloc: 100, complexity: 10.0, churn: 20, contributors: 3, heatmap_score: 0.0,
        },
    ];

    let fan_in: HashMap<FileId, usize> = HashMap::new();
    codeilus_metrics::heatmap::compute_heatmap(&mut metrics, &fan_in);

    // complex.py should have the highest score
    let complex = metrics.iter().find(|m| m.path == "complex.py").unwrap();
    let simple = metrics.iter().find(|m| m.path == "simple.py").unwrap();
    assert!(
        complex.heatmap_score > simple.heatmap_score,
        "complex.py ({}) should score higher than simple.py ({})",
        complex.heatmap_score, simple.heatmap_score
    );
}

#[test]
fn heatmap_normalization() {
    let mut metrics = vec![
        FileMetrics {
            file_id: FileId(1), path: "a.py".to_string(),
            sloc: 10, complexity: 1.0, churn: 0, contributors: 0, heatmap_score: 0.0,
        },
        FileMetrics {
            file_id: FileId(2), path: "b.py".to_string(),
            sloc: 100, complexity: 20.0, churn: 50, contributors: 5, heatmap_score: 0.0,
        },
    ];

    let fan_in: HashMap<FileId, usize> = HashMap::new();
    codeilus_metrics::heatmap::compute_heatmap(&mut metrics, &fan_in);

    for m in &metrics {
        assert!(
            m.heatmap_score >= 0.0 && m.heatmap_score <= 1.0,
            "Heatmap score should be 0.0-1.0, got {} for {}",
            m.heatmap_score, m.path
        );
    }
}

// --- Integration Test ---

#[test]
fn compute_metrics_integration() {
    let kg = make_simple_graph();

    // Create temp files for the parsed files to read from
    let dir = tempfile::tempdir().unwrap();
    let py_path = dir.path().join("main.py");
    std::fs::write(&py_path, "def func_a():\n    pass\n\ndef func_b():\n    pass\n").unwrap();
    let rs_path = dir.path().join("utils.rs");
    std::fs::write(&rs_path, "fn func_c() {\n    // code\n}\n").unwrap();

    let parsed_files = vec![
        make_parsed_file(
            py_path.to_str().unwrap(),
            Language::Python,
            vec![
                make_symbol("func_a", SymbolKind::Function, 1, 2),
                make_symbol("func_b", SymbolKind::Function, 4, 5),
            ],
        ),
        make_parsed_file(
            rs_path.to_str().unwrap(),
            Language::Rust,
            vec![make_symbol("func_c", SymbolKind::Function, 1, 3)],
        ),
    ];

    let report = codeilus_metrics::compute_metrics(&parsed_files, &kg, dir.path()).unwrap();

    // File metrics
    assert_eq!(report.file_metrics.len(), 2);
    assert!(report.file_metrics.iter().all(|m| m.sloc > 0));

    // Symbol metrics
    assert!(!report.symbol_metrics.is_empty());

    // Community metrics
    assert!(!report.community_metrics.is_empty());

    // Repo metrics
    assert_eq!(report.repo_metrics.total_files, 2);
    assert!(report.repo_metrics.total_sloc > 0);
    assert!(report.repo_metrics.total_symbols > 0);
    assert!(!report.repo_metrics.language_breakdown.is_empty());
}
