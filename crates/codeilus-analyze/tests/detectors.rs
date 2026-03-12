use std::path::PathBuf;

use codeilus_analyze::{
    circular_deps, god_class, long_method, security, test_gap, PatternKind, Severity,
};
use codeilus_core::ids::SymbolId;
use codeilus_core::{Confidence, EdgeKind, Language, SymbolKind};
use codeilus_graph::{GraphEdge, GraphNode, KnowledgeGraph};
use codeilus_parse::{ParsedFile, Symbol};
use petgraph::graph::DiGraph;
use std::collections::HashMap;

fn make_file(path: &str, lang: Language, symbols: Vec<Symbol>) -> ParsedFile {
    ParsedFile {
        path: PathBuf::from(path),
        language: lang,
        sloc: symbols.len() * 10,
        symbols,
        imports: Vec::new(),
        calls: Vec::new(),
        heritage: Vec::new(),
    }
}

fn make_method(name: &str, start: i64, end: i64) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind: SymbolKind::Method,
        start_line: start,
        end_line: end,
        signature: None,
    }
}

fn make_function(name: &str, start: i64, end: i64) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind: SymbolKind::Function,
        start_line: start,
        end_line: end,
        signature: None,
    }
}

fn empty_graph() -> KnowledgeGraph {
    KnowledgeGraph {
        graph: DiGraph::new(),
        node_index: HashMap::new(),
        communities: Vec::new(),
        processes: Vec::new(),
        entry_points: Vec::new(),
    }
}

// ── God Class ─────────────────────────────────────────────────

#[test]
fn god_class_detected() {
    let mut symbols = vec![Symbol {
        name: "BigClass".to_string(),
        kind: SymbolKind::Class,
        start_line: 1,
        end_line: 1000,
        signature: None,
    }];
    // Add 25 methods within the class's range
    for i in 0..25 {
        symbols.push(make_method(
            &format!("method_{i}"),
            (i * 10 + 2) as i64,
            (i * 10 + 10) as i64,
        ));
    }
    let files = vec![make_file("src/big.py", Language::Python, symbols)];
    let findings = god_class::detect(&files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, PatternKind::GodClass);
    assert_eq!(findings[0].severity, Severity::Warning);
}

#[test]
fn god_class_not_triggered() {
    let mut symbols = vec![Symbol {
        name: "SmallClass".to_string(),
        kind: SymbolKind::Class,
        start_line: 1,
        end_line: 500,
        signature: None,
    }];
    for i in 0..10 {
        symbols.push(make_method(
            &format!("method_{i}"),
            (i * 10 + 2) as i64,
            (i * 10 + 10) as i64,
        ));
    }
    let files = vec![make_file("src/small.py", Language::Python, symbols)];
    let findings = god_class::detect(&files).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn god_class_severe() {
    let mut symbols = vec![Symbol {
        name: "HugeClass".to_string(),
        kind: SymbolKind::Class,
        start_line: 1,
        end_line: 2000,
        signature: None,
    }];
    for i in 0..35 {
        symbols.push(make_method(
            &format!("method_{i}"),
            (i * 10 + 2) as i64,
            (i * 10 + 10) as i64,
        ));
    }
    let files = vec![make_file("src/huge.py", Language::Python, symbols)];
    let findings = god_class::detect(&files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Error);
}

// ── Long Method ───────────────────────────────────────────────

#[test]
fn long_method_50_lines() {
    // Exactly 50 lines → should NOT trigger (must exceed)
    let files = vec![make_file(
        "src/exact.py",
        Language::Python,
        vec![make_function("short_fn", 1, 51)],
    )];
    let findings = long_method::detect(&files).unwrap();
    assert!(findings.is_empty());
}

#[test]
fn long_method_100_lines() {
    let files = vec![make_file(
        "src/mid.py",
        Language::Python,
        vec![make_function("mid_fn", 1, 102)],
    )];
    let findings = long_method::detect(&files).unwrap();
    assert_eq!(findings.len(), 1);
    // 101 lines → Warning
    assert_eq!(findings[0].severity, Severity::Warning);
}

#[test]
fn long_method_250_lines() {
    let files = vec![make_file(
        "src/huge.py",
        Language::Python,
        vec![make_function("huge_fn", 1, 251)],
    )];
    let findings = long_method::detect(&files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Error);
}

// ── Circular Dependencies ─────────────────────────────────────

#[test]
fn circular_dep_simple() {
    let mut graph = DiGraph::new();
    let a = graph.add_node(GraphNode {
        symbol_id: SymbolId(1),
        file_id: codeilus_core::ids::FileId(1),
        name: "A".to_string(),
        kind: "Module".to_string(),
        community_id: None,
    });
    let b = graph.add_node(GraphNode {
        symbol_id: SymbolId(2),
        file_id: codeilus_core::ids::FileId(1),
        name: "B".to_string(),
        kind: "Module".to_string(),
        community_id: None,
    });
    graph.add_edge(a, b, GraphEdge {
        kind: EdgeKind::Imports,
        confidence: Confidence::certain(),
    });
    graph.add_edge(b, a, GraphEdge {
        kind: EdgeKind::Imports,
        confidence: Confidence::certain(),
    });

    let kg = KnowledgeGraph {
        graph,
        node_index: HashMap::from([(SymbolId(1), a), (SymbolId(2), b)]),
        communities: Vec::new(),
        processes: Vec::new(),
        entry_points: Vec::new(),
    };

    let findings = circular_deps::detect(&kg).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, PatternKind::CircularDependency);
    assert_eq!(findings[0].severity, Severity::Warning);
}

#[test]
fn circular_dep_three_node() {
    let mut graph = DiGraph::new();
    let a = graph.add_node(GraphNode {
        symbol_id: SymbolId(1),
        file_id: codeilus_core::ids::FileId(1),
        name: "A".to_string(),
        kind: "Module".to_string(),
        community_id: None,
    });
    let b = graph.add_node(GraphNode {
        symbol_id: SymbolId(2),
        file_id: codeilus_core::ids::FileId(1),
        name: "B".to_string(),
        kind: "Module".to_string(),
        community_id: None,
    });
    let c = graph.add_node(GraphNode {
        symbol_id: SymbolId(3),
        file_id: codeilus_core::ids::FileId(1),
        name: "C".to_string(),
        kind: "Module".to_string(),
        community_id: None,
    });
    graph.add_edge(a, b, GraphEdge {
        kind: EdgeKind::Imports,
        confidence: Confidence::certain(),
    });
    graph.add_edge(b, c, GraphEdge {
        kind: EdgeKind::Imports,
        confidence: Confidence::certain(),
    });
    graph.add_edge(c, a, GraphEdge {
        kind: EdgeKind::Imports,
        confidence: Confidence::certain(),
    });

    let kg = KnowledgeGraph {
        graph,
        node_index: HashMap::from([(SymbolId(1), a), (SymbolId(2), b), (SymbolId(3), c)]),
        communities: Vec::new(),
        processes: Vec::new(),
        entry_points: Vec::new(),
    };

    let findings = circular_deps::detect(&kg).unwrap();
    assert_eq!(findings.len(), 1);
    assert!(findings[0].message.contains("A"));
    assert!(findings[0].message.contains("B"));
    assert!(findings[0].message.contains("C"));
}

#[test]
fn circular_dep_none() {
    // DAG with no cycles
    let mut graph = DiGraph::new();
    let a = graph.add_node(GraphNode {
        symbol_id: SymbolId(1),
        file_id: codeilus_core::ids::FileId(1),
        name: "A".to_string(),
        kind: "Module".to_string(),
        community_id: None,
    });
    let b = graph.add_node(GraphNode {
        symbol_id: SymbolId(2),
        file_id: codeilus_core::ids::FileId(1),
        name: "B".to_string(),
        kind: "Module".to_string(),
        community_id: None,
    });
    graph.add_edge(a, b, GraphEdge {
        kind: EdgeKind::Imports,
        confidence: Confidence::certain(),
    });

    let kg = KnowledgeGraph {
        graph,
        node_index: HashMap::from([(SymbolId(1), a), (SymbolId(2), b)]),
        communities: Vec::new(),
        processes: Vec::new(),
        entry_points: Vec::new(),
    };

    let findings = circular_deps::detect(&kg).unwrap();
    assert!(findings.is_empty());
}

// ── Security ──────────────────────────────────────────────────

#[test]
fn security_eval() {
    let files = vec![
        ("app.py".to_string(), "result = eval(user_input)".to_string()),
    ];
    let findings = security::detect_in_content(&files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, PatternKind::SecurityHotspot);
    assert_eq!(findings[0].severity, Severity::Warning);
}

#[test]
fn security_hardcoded_secret() {
    let files = vec![(
        "config.py".to_string(),
        r#"password = "abc123""#.to_string(),
    )];
    let findings = security::detect_in_content(&files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].severity, Severity::Error);
    assert!(findings[0].message.contains("Hardcoded secret"));
}

#[test]
fn security_sql_injection() {
    let files = vec![(
        "db.py".to_string(),
        r#"query = f"SELECT * FROM {table}""#.to_string(),
    )];
    let findings = security::detect_in_content(&files).unwrap();
    assert!(!findings.is_empty());
    assert!(findings.iter().any(|f| f.message.contains("SQL injection")));
}

#[test]
fn security_clean_file() {
    let files = vec![(
        "clean.py".to_string(),
        "def hello():\n    return 'world'\n".to_string(),
    )];
    let findings = security::detect_in_content(&files).unwrap();
    assert!(findings.is_empty());
}

// ── Test Gap ──────────────────────────────────────────────────

#[test]
fn test_gap_missing() {
    // src/parser.py with 10 symbols, no test file in the list
    let mut symbols = Vec::new();
    for i in 0..10 {
        symbols.push(make_function(&format!("func_{i}"), i * 5, i * 5 + 4));
    }
    let files = vec![make_file("src/parser.py", Language::Python, symbols)];
    let findings = test_gap::detect(&files).unwrap();
    assert_eq!(findings.len(), 1);
    assert_eq!(findings[0].kind, PatternKind::TestGap);
    assert_eq!(findings[0].severity, Severity::Info);
}

#[test]
fn test_gap_covered() {
    let mut symbols = Vec::new();
    for i in 0..10 {
        symbols.push(make_function(&format!("func_{i}"), i * 5, i * 5 + 4));
    }
    let files = vec![
        make_file("src/parser.py", Language::Python, symbols),
        make_file("src/test_parser.py", Language::Python, Vec::new()),
    ];
    let findings = test_gap::detect(&files).unwrap();
    // src/parser.py has a matching test file → no finding
    assert!(findings.is_empty());
}

// ── Integration ───────────────────────────────────────────────

#[test]
fn analyze_integration() {
    // Build parsed files with a god class + long method
    let mut symbols = vec![Symbol {
        name: "FatClass".to_string(),
        kind: SymbolKind::Class,
        start_line: 1,
        end_line: 2000,
        signature: None,
    }];
    for i in 0..25 {
        symbols.push(make_method(
            &format!("m_{i}"),
            (i * 10 + 2) as i64,
            (i * 10 + 10) as i64,
        ));
    }
    // Also a long function
    symbols.push(make_function("big_fn", 2001, 2252));

    let files = vec![make_file("src/fat.py", Language::Python, symbols)];
    let graph = empty_graph();

    let findings = codeilus_analyze::analyze(&files, &graph).unwrap();
    // Should have at least: 1 god class + 1 long method + 1 test gap
    assert!(findings.len() >= 3);
    assert!(findings.iter().any(|f| f.kind == PatternKind::GodClass));
    assert!(findings.iter().any(|f| f.kind == PatternKind::LongMethod));
    assert!(findings.iter().any(|f| f.kind == PatternKind::TestGap));
}
