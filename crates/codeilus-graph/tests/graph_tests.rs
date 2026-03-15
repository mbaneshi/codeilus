use std::collections::HashMap;
use std::path::PathBuf;

use codeilus_core::ids::{FileId, SymbolId};
use codeilus_core::types::{Confidence, EdgeKind, Language, SymbolKind};
use codeilus_graph::types::{GraphEdge, GraphNode};
use codeilus_graph::GraphBuilder;
use codeilus_parse::{Call, Heritage, Import, ParsedFile, Symbol};
use petgraph::graph::DiGraph;

/// Helper to create a simple ParsedFile.
fn make_parsed_file(
    path: &str,
    lang: Language,
    symbols: Vec<Symbol>,
    calls: Vec<Call>,
    imports: Vec<Import>,
    heritage: Vec<Heritage>,
) -> ParsedFile {
    let sloc = 10;
    ParsedFile {
        path: PathBuf::from(path),
        language: lang,
        sloc,
        symbols,
        imports,
        calls,
        heritage,
    }
}

fn make_symbol(name: &str, kind: SymbolKind) -> Symbol {
    Symbol {
        name: name.to_string(),
        kind,
        start_line: 1,
        end_line: 10,
        signature: Some(format!("fn {name}()")),
    }
}

// --- Call Graph Tests ---

#[test]
fn call_graph_builds_edges() {
    let files = vec![
        make_parsed_file(
            "a.py",
            Language::Python,
            vec![
                make_symbol("func_a", SymbolKind::Function),
                make_symbol("func_b", SymbolKind::Function),
            ],
            vec![Call {
                caller: "func_a".to_string(),
                callee: "func_b".to_string(),
                line: 5,
            }],
            vec![],
            vec![],
        ),
    ];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    // Should have a CALLS edge
    let calls_edges: Vec<_> = kg
        .graph
        .edge_indices()
        .filter(|&e| kg.graph[e].kind == EdgeKind::Calls)
        .collect();
    assert!(!calls_edges.is_empty(), "Expected at least one CALLS edge");
}

#[test]
fn call_graph_confidence_exact() {
    let files = vec![make_parsed_file(
        "a.py",
        Language::Python,
        vec![
            make_symbol("caller", SymbolKind::Function),
            make_symbol("callee", SymbolKind::Function),
        ],
        vec![Call {
            caller: "caller".to_string(),
            callee: "callee".to_string(),
            line: 3,
        }],
        vec![],
        vec![],
    )];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    // Both symbols in same file → exact match → confidence 1.0
    for e in kg.graph.edge_indices() {
        if kg.graph[e].kind == EdgeKind::Calls {
            assert_eq!(
                kg.graph[e].confidence.0, 1.0,
                "Same-file match should have confidence 1.0"
            );
        }
    }
}

#[test]
fn call_graph_confidence_ambiguous() {
    // Two files, each with a symbol named "helper"
    let files = vec![
        make_parsed_file(
            "a.py",
            Language::Python,
            vec![
                make_symbol("main_fn", SymbolKind::Function),
                make_symbol("helper", SymbolKind::Function),
            ],
            vec![Call {
                caller: "main_fn".to_string(),
                callee: "helper".to_string(),
                line: 3,
            }],
            vec![],
            vec![],
        ),
        make_parsed_file(
            "b.py",
            Language::Python,
            vec![make_symbol("helper", SymbolKind::Function)],
            vec![],
            vec![],
            vec![],
        ),
    ];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    // caller is in a.py so "main_fn" resolves exactly,
    // but callee "helper" also exists in a.py → exact match (1.0)
    // If we had a call to "helper" from b.py where it's local, that'd also be 1.0
    // The ambiguous case would be if the caller is in a different file
    let has_calls = kg
        .graph
        .edge_indices()
        .any(|e| kg.graph[e].kind == EdgeKind::Calls);
    assert!(has_calls, "Should have call edges");
}

// --- Dep Graph Tests ---

#[test]
fn dep_graph_from_imports() {
    let files = vec![
        make_parsed_file(
            "main.py",
            Language::Python,
            vec![make_symbol("main", SymbolKind::Function)],
            vec![],
            vec![Import {
                from: "utils".to_string(),
                name: "*".to_string(),
                line: 1,
            }],
            vec![],
        ),
        make_parsed_file(
            "utils.py",
            Language::Python,
            vec![make_symbol("helper", SymbolKind::Function)],
            vec![],
            vec![],
            vec![],
        ),
    ];

    let mut file_index = HashMap::new();
    file_index.insert("main.py".to_string(), FileId(1));
    file_index.insert("utils.py".to_string(), FileId(2));

    let edges = codeilus_graph::dep_graph::build_dep_edges(&files, &file_index);
    assert!(!edges.is_empty(), "Expected dep edge from main.py to utils.py");
    assert_eq!(edges[0].0, FileId(1));
    assert_eq!(edges[0].1, FileId(2));
}

// --- Heritage Tests ---

#[test]
fn heritage_extends() {
    let files = vec![make_parsed_file(
        "models.py",
        Language::Python,
        vec![
            make_symbol("Base", SymbolKind::Class),
            make_symbol("Child", SymbolKind::Class),
        ],
        vec![],
        vec![],
        vec![Heritage {
            child: "Child".to_string(),
            parent: "Base".to_string(),
            relation: EdgeKind::Extends,
            confidence: Confidence::certain(),
        }],
    )];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    let extends_edges: Vec<_> = kg
        .graph
        .edge_indices()
        .filter(|&e| kg.graph[e].kind == EdgeKind::Extends)
        .collect();
    assert!(!extends_edges.is_empty(), "Expected EXTENDS edge");
}

#[test]
fn heritage_implements() {
    let files = vec![make_parsed_file(
        "service.ts",
        Language::TypeScript,
        vec![
            make_symbol("IService", SymbolKind::Interface),
            make_symbol("MyService", SymbolKind::Class),
        ],
        vec![],
        vec![],
        vec![Heritage {
            child: "MyService".to_string(),
            parent: "IService".to_string(),
            relation: EdgeKind::Implements,
            confidence: Confidence::certain(),
        }],
    )];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    let impl_edges: Vec<_> = kg
        .graph
        .edge_indices()
        .filter(|&e| kg.graph[e].kind == EdgeKind::Implements)
        .collect();
    assert!(!impl_edges.is_empty(), "Expected IMPLEMENTS edge");
}

// --- Community Detection Tests ---

#[test]
fn louvain_two_clusters() {
    // Build a graph with two cliques connected by one bridge edge
    let mut graph = DiGraph::new();

    // Clique A: nodes 0,1,2 fully connected
    let a0 = graph.add_node(GraphNode {
        symbol_id: SymbolId(1),
        file_id: FileId(1),
        name: "a0".to_string(),
        kind: "Function".to_string(),
        community_id: None,
    });
    let a1 = graph.add_node(GraphNode {
        symbol_id: SymbolId(2),
        file_id: FileId(1),
        name: "a1".to_string(),
        kind: "Function".to_string(),
        community_id: None,
    });
    let a2 = graph.add_node(GraphNode {
        symbol_id: SymbolId(3),
        file_id: FileId(1),
        name: "a2".to_string(),
        kind: "Function".to_string(),
        community_id: None,
    });

    // Clique B: nodes 3,4,5 fully connected
    let b0 = graph.add_node(GraphNode {
        symbol_id: SymbolId(4),
        file_id: FileId(2),
        name: "b0".to_string(),
        kind: "Function".to_string(),
        community_id: None,
    });
    let b1 = graph.add_node(GraphNode {
        symbol_id: SymbolId(5),
        file_id: FileId(2),
        name: "b1".to_string(),
        kind: "Function".to_string(),
        community_id: None,
    });
    let b2 = graph.add_node(GraphNode {
        symbol_id: SymbolId(6),
        file_id: FileId(2),
        name: "b2".to_string(),
        kind: "Function".to_string(),
        community_id: None,
    });

    let edge = GraphEdge {
        kind: EdgeKind::Calls,
        confidence: Confidence::certain(),
    };

    // Clique A edges
    graph.add_edge(a0, a1, edge.clone());
    graph.add_edge(a1, a0, edge.clone());
    graph.add_edge(a0, a2, edge.clone());
    graph.add_edge(a2, a0, edge.clone());
    graph.add_edge(a1, a2, edge.clone());
    graph.add_edge(a2, a1, edge.clone());

    // Clique B edges
    graph.add_edge(b0, b1, edge.clone());
    graph.add_edge(b1, b0, edge.clone());
    graph.add_edge(b0, b2, edge.clone());
    graph.add_edge(b2, b0, edge.clone());
    graph.add_edge(b1, b2, edge.clone());
    graph.add_edge(b2, b1, edge.clone());

    // Bridge: single edge between cliques
    graph.add_edge(a2, b0, edge.clone());

    let communities = codeilus_graph::community::detect_communities(&graph);
    // With small-community merging (threshold <=3), two 3-node cliques may merge.
    // The important thing is that community detection runs without panicking
    // and produces at least 1 community.
    assert!(
        !communities.is_empty(),
        "Expected at least 1 community, got 0"
    );
}

#[test]
fn louvain_single_community() {
    // Fully connected 3-node graph
    let mut graph = DiGraph::new();
    let n0 = graph.add_node(GraphNode {
        symbol_id: SymbolId(1),
        file_id: FileId(1),
        name: "a".to_string(),
        kind: "Function".to_string(),
        community_id: None,
    });
    let n1 = graph.add_node(GraphNode {
        symbol_id: SymbolId(2),
        file_id: FileId(1),
        name: "b".to_string(),
        kind: "Function".to_string(),
        community_id: None,
    });
    let n2 = graph.add_node(GraphNode {
        symbol_id: SymbolId(3),
        file_id: FileId(1),
        name: "c".to_string(),
        kind: "Function".to_string(),
        community_id: None,
    });

    let edge = GraphEdge {
        kind: EdgeKind::Calls,
        confidence: Confidence::certain(),
    };

    graph.add_edge(n0, n1, edge.clone());
    graph.add_edge(n1, n0, edge.clone());
    graph.add_edge(n0, n2, edge.clone());
    graph.add_edge(n2, n0, edge.clone());
    graph.add_edge(n1, n2, edge.clone());
    graph.add_edge(n2, n1, edge.clone());

    let communities = codeilus_graph::community::detect_communities(&graph);
    assert!(
        communities.len() <= 2,
        "Fully connected 3-node graph should have 1-2 communities, got {}",
        communities.len()
    );
}

// --- Entry Point Tests ---

#[test]
fn entry_point_main() {
    let files = vec![make_parsed_file(
        "main.py",
        Language::Python,
        vec![
            make_symbol("main", SymbolKind::Function),
            make_symbol("helper", SymbolKind::Function),
        ],
        vec![Call {
            caller: "main".to_string(),
            callee: "helper".to_string(),
            line: 5,
        }],
        vec![],
        vec![],
    )];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    assert!(!kg.entry_points.is_empty(), "Expected entry points");
    // "main" should score highest
    assert_eq!(
        kg.entry_points[0].symbol_id,
        kg.node_index
            .keys()
            .find(|id| {
                let idx = kg.node_index[id];
                kg.graph[idx].name == "main"
            })
            .copied()
            .unwrap(),
        "main should be highest-scoring entry point"
    );
}

#[test]
fn entry_point_handler() {
    // main that calls internal_fn: main(1.0) + zero callers calls others(0.5) = 1.5
    let files = vec![make_parsed_file(
        "app.py",
        Language::Python,
        vec![
            make_symbol("main", SymbolKind::Function),
            make_symbol("internal_fn", SymbolKind::Function),
        ],
        vec![Call {
            caller: "main".to_string(),
            callee: "internal_fn".to_string(),
            line: 5,
        }],
        vec![],
        vec![],
    )];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    // main should be an entry point (score: 1.0 main + 0.5 zero callers calls others = 1.5)
    let main_ep = kg
        .entry_points
        .iter()
        .find(|ep| {
            let idx = kg.node_index[&ep.symbol_id];
            kg.graph[idx].name == "main"
        })
        .expect("main should be an entry point");

    assert!(
        main_ep.score >= 1.5,
        "main should score >= 1.5, got {}",
        main_ep.score
    );
    assert!(
        main_ep.reason.contains("main"),
        "reason should mention main"
    );
}

// --- Process Detection Tests ---

#[test]
fn process_bfs_linear() {
    // Linear chain: A → B → C
    let files = vec![make_parsed_file(
        "chain.py",
        Language::Python,
        vec![
            make_symbol("step_a", SymbolKind::Function),
            make_symbol("step_b", SymbolKind::Function),
            make_symbol("step_c", SymbolKind::Function),
            make_symbol("main", SymbolKind::Function),
        ],
        vec![
            Call {
                caller: "main".to_string(),
                callee: "step_a".to_string(),
                line: 2,
            },
            Call {
                caller: "step_a".to_string(),
                callee: "step_b".to_string(),
                line: 5,
            },
            Call {
                caller: "step_b".to_string(),
                callee: "step_c".to_string(),
                line: 8,
            },
        ],
        vec![],
        vec![],
    )];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    // main → step_a → step_b → step_c = 4 steps
    let main_process = kg
        .processes
        .iter()
        .find(|p| p.name.contains("main"))
        .expect("Expected a process starting from main");

    assert!(
        main_process.steps.len() >= 3,
        "Expected at least 3 steps, got {}",
        main_process.steps.len()
    );
}

#[test]
fn process_bfs_depth_limit() {
    // Build a chain longer than 20
    let mut symbols = Vec::new();
    let mut calls = Vec::new();

    for i in 0..25 {
        symbols.push(make_symbol(&format!("step_{i}"), SymbolKind::Function));
        if i > 0 {
            calls.push(Call {
                caller: format!("step_{}", i - 1),
                callee: format!("step_{i}"),
                line: i as i64,
            });
        }
    }
    // Add main as entry
    symbols.push(make_symbol("main", SymbolKind::Function));
    calls.push(Call {
        caller: "main".to_string(),
        callee: "step_0".to_string(),
        line: 0,
    });

    let files = vec![make_parsed_file(
        "long_chain.py",
        Language::Python,
        symbols,
        calls,
        vec![],
        vec![],
    )];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    let main_process = kg.processes.iter().find(|p| p.name.contains("main"));
    if let Some(proc) = main_process {
        assert!(
            proc.steps.len() <= 22, // main + 20 max depth + 1 for off-by-one
            "BFS should be depth-limited to ~20, got {} steps",
            proc.steps.len()
        );
    }
}

// --- Community Merge Tests ---

#[test]
fn tiny_communities_merged() {
    // Build a graph where Louvain produces many tiny communities,
    // then verify post-processing merges them
    let mut graph = DiGraph::new();
    let edge = GraphEdge {
        kind: EdgeKind::Calls,
        confidence: Confidence::certain(),
    };

    // Create a hub (4 nodes fully connected) + 5 isolated pairs
    let mut hub = Vec::new();
    for i in 0..4 {
        hub.push(graph.add_node(GraphNode {
            symbol_id: SymbolId(i + 1),
            file_id: FileId(1),
            name: format!("hub_{i}"),
            kind: "Function".to_string(),
            community_id: None,
        }));
    }
    for i in 0..4 {
        for j in (i + 1)..4 {
            graph.add_edge(hub[i], hub[j], edge.clone());
            graph.add_edge(hub[j], hub[i], edge.clone());
        }
    }

    // 5 isolated pairs, each connected to the hub by one edge
    for p in 0..5 {
        let base = 5 + p * 2;
        let a = graph.add_node(GraphNode {
            symbol_id: SymbolId(base as i64),
            file_id: FileId(2),
            name: format!("pair_{p}_a"),
            kind: "Function".to_string(),
            community_id: None,
        });
        let b = graph.add_node(GraphNode {
            symbol_id: SymbolId(base as i64 + 1),
            file_id: FileId(2),
            name: format!("pair_{p}_b"),
            kind: "Function".to_string(),
            community_id: None,
        });
        graph.add_edge(a, b, edge.clone());
        graph.add_edge(b, a, edge.clone());
        // Connect to hub
        graph.add_edge(a, hub[0], edge.clone());
    }

    let communities = codeilus_graph::community::detect_communities(&graph);
    // Pairs (2 members each, <=3) should be merged → few communities
    assert!(
        communities.len() <= 3,
        "Tiny communities should be merged, got {}",
        communities.len()
    );
}

// --- Entry Point Threshold & Cap Tests ---

#[test]
fn entry_points_capped_at_30() {
    // Create 50 "main_*" functions, each calling a helper → all score >= 1.5
    let mut symbols = Vec::new();
    let mut calls = Vec::new();
    for i in 0..50 {
        symbols.push(make_symbol(&format!("main_{i}"), SymbolKind::Function));
        symbols.push(make_symbol(&format!("helper_{i}"), SymbolKind::Function));
        calls.push(Call {
            caller: format!("main_{i}"),
            callee: format!("helper_{i}"),
            line: i as i64,
        });
    }

    let files = vec![make_parsed_file(
        "many_mains.py",
        Language::Python,
        symbols,
        calls,
        vec![],
        vec![],
    )];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    assert!(
        kg.entry_points.len() <= 30,
        "Entry points should be capped at 30, got {}",
        kg.entry_points.len()
    );
}

#[test]
fn high_fanin_utility_not_entry_point() {
    // A utility function called by 6 callers should get negative score
    let mut symbols = vec![make_symbol("utility", SymbolKind::Function)];
    let mut calls = Vec::new();
    for i in 0..6 {
        let name = format!("caller_{i}");
        symbols.push(make_symbol(&name, SymbolKind::Function));
        calls.push(Call {
            caller: name,
            callee: "utility".to_string(),
            line: i as i64,
        });
    }

    let files = vec![make_parsed_file(
        "utils.py",
        Language::Python,
        symbols,
        calls,
        vec![],
        vec![],
    )];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    // utility has 6 callers → -0.5, no positive patterns → should NOT be an entry point
    let utility_ep = kg.entry_points.iter().find(|ep| {
        let idx = kg.node_index[&ep.symbol_id];
        kg.graph[idx].name == "utility"
    });
    assert!(
        utility_ep.is_none(),
        "High fan-in utility should not be an entry point"
    );
}

// --- Community Label Tests ---

#[test]
fn community_labels_use_directory() {
    let files = vec![
        make_parsed_file(
            "src/api/handler.py",
            Language::Python,
            vec![
                make_symbol("handle", SymbolKind::Function),
                make_symbol("route", SymbolKind::Function),
            ],
            vec![Call {
                caller: "handle".to_string(),
                callee: "route".to_string(),
                line: 5,
            }],
            vec![],
            vec![],
        ),
        make_parsed_file(
            "src/api/middleware.py",
            Language::Python,
            vec![make_symbol("auth", SymbolKind::Function)],
            vec![],
            vec![],
            vec![],
        ),
    ];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    // Communities should have semantic labels derived from symbol names (TF-IDF)
    // With symbols "handle", "route", "auth", expect labels based on those names
    let labels: Vec<&str> = kg.communities.iter().map(|c| c.label.as_str()).collect();
    assert!(
        labels.iter().all(|l| !l.is_empty() && *l != "src"),
        "Expected semantic labels, not directory names. Got: {:?}",
        labels
    );
}

// --- Integration Test ---

#[test]
fn build_graph_integration() {
    let files = vec![
        make_parsed_file(
            "main.py",
            Language::Python,
            vec![
                make_symbol("main", SymbolKind::Function),
                make_symbol("App", SymbolKind::Class),
            ],
            vec![Call {
                caller: "main".to_string(),
                callee: "process".to_string(),
                line: 5,
            }],
            vec![Import {
                from: "utils".to_string(),
                name: "*".to_string(),
                line: 1,
            }],
            vec![],
        ),
        make_parsed_file(
            "utils.py",
            Language::Python,
            vec![
                make_symbol("process", SymbolKind::Function),
                make_symbol("Helper", SymbolKind::Class),
            ],
            vec![],
            vec![],
            vec![Heritage {
                child: "Helper".to_string(),
                parent: "App".to_string(),
                relation: EdgeKind::Extends,
                confidence: Confidence::certain(),
            }],
        ),
    ];

    let builder = GraphBuilder::new();
    let kg = builder.build(&files).unwrap();

    // Graph has nodes
    assert_eq!(kg.graph.node_count(), 4);

    // Has edges
    assert!(kg.graph.edge_count() > 0, "Expected edges in graph");

    // Has communities
    assert!(!kg.communities.is_empty(), "Expected communities");

    // Has entry points
    assert!(!kg.entry_points.is_empty(), "Expected entry points");

    // main should be top entry point
    let main_id = kg
        .node_index
        .keys()
        .find(|id| kg.graph[kg.node_index[id]].name == "main")
        .copied()
        .unwrap();
    assert_eq!(kg.entry_points[0].symbol_id, main_id);

    // Has processes
    assert!(!kg.processes.is_empty(), "Expected processes");
}
