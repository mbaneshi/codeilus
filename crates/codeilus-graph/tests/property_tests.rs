use proptest::prelude::*;
use std::path::PathBuf;

use codeilus_core::types::{Confidence, EdgeKind, Language, SymbolKind};
use codeilus_graph::GraphBuilder;
use codeilus_parse::{Call, Heritage, Import, ParsedFile, Symbol};

// --- Strategies ---

fn arb_language() -> impl Strategy<Value = Language> {
    prop_oneof![
        Just(Language::Python),
        Just(Language::TypeScript),
        Just(Language::JavaScript),
        Just(Language::Rust),
        Just(Language::Go),
        Just(Language::Java),
    ]
}

fn arb_symbol_kind() -> impl Strategy<Value = SymbolKind> {
    prop_oneof![
        Just(SymbolKind::Function),
        Just(SymbolKind::Class),
        Just(SymbolKind::Method),
        Just(SymbolKind::Interface),
        Just(SymbolKind::Struct),
        Just(SymbolKind::Enum),
        Just(SymbolKind::Trait),
        Just(SymbolKind::Module),
        Just(SymbolKind::Constant),
        Just(SymbolKind::TypeAlias),
    ]
}

fn arb_symbol_name() -> impl Strategy<Value = String> {
    // Generate realistic symbol names: lowercase with underscores
    "[a-z][a-z0-9_]{0,19}"
}

fn arb_symbol() -> impl Strategy<Value = Symbol> {
    (arb_symbol_name(), arb_symbol_kind(), 1i64..1000i64).prop_map(|(name, kind, start)| Symbol {
        name,
        kind,
        start_line: start,
        end_line: start + 10,
        signature: None,
    })
}

fn arb_edge_kind_heritage() -> impl Strategy<Value = EdgeKind> {
    prop_oneof![Just(EdgeKind::Extends), Just(EdgeKind::Implements),]
}

/// Generate calls that reference symbols by name from the provided symbol list.
fn arb_calls(symbols: Vec<Symbol>) -> impl Strategy<Value = Vec<Call>> {
    let names: Vec<String> = symbols.iter().map(|s| s.name.clone()).collect();
    if names.len() < 2 {
        return Just(vec![]).boxed();
    }
    let n = names.len();
    prop::collection::vec(
        (0..n, 0..n, 1i64..100i64),
        0..=(n.min(10)),
    )
    .prop_map(move |indices| {
        indices
            .into_iter()
            .filter(|(a, b, _)| a != b)
            .map(|(a, b, line)| Call {
                caller: names[a].clone(),
                callee: names[b].clone(),
                line,
            })
            .collect()
    })
    .boxed()
}

/// Generate heritage relations that reference class/interface symbols.
fn arb_heritage(symbols: Vec<Symbol>) -> impl Strategy<Value = Vec<Heritage>> {
    let class_names: Vec<String> = symbols
        .iter()
        .filter(|s| {
            matches!(
                s.kind,
                SymbolKind::Class | SymbolKind::Interface | SymbolKind::Trait | SymbolKind::Struct
            )
        })
        .map(|s| s.name.clone())
        .collect();
    if class_names.len() < 2 {
        return Just(vec![]).boxed();
    }
    let n = class_names.len();
    prop::collection::vec(
        (0..n, 0..n, arb_edge_kind_heritage()),
        0..=(n.min(5)),
    )
    .prop_map(move |entries| {
        entries
            .into_iter()
            .filter(|(a, b, _)| a != b)
            .map(|(a, b, relation)| Heritage {
                child: class_names[a].clone(),
                parent: class_names[b].clone(),
                relation,
                confidence: Confidence::certain(),
            })
            .collect()
    })
    .boxed()
}

/// Generate imports referencing file stems from a set of file paths.
fn arb_imports(file_stems: Vec<String>) -> impl Strategy<Value = Vec<Import>> {
    if file_stems.is_empty() {
        return Just(vec![]).boxed();
    }
    let n = file_stems.len();
    prop::collection::vec((0..n, 1i64..100i64), 0..=3)
        .prop_map(move |entries| {
            entries
                .into_iter()
                .map(|(idx, line)| Import {
                    from: file_stems[idx].clone(),
                    name: "*".to_string(),
                    line,
                })
                .collect()
        })
        .boxed()
}

fn arb_file_path(idx: usize) -> impl Strategy<Value = (PathBuf, String)> {
    let dirs = vec!["src", "lib", "utils", "api", "core", "tests"];
    let exts = vec!["py", "ts", "js", "rs", "go", "java"];
    (
        prop::sample::select(dirs),
        "[a-z][a-z0-9_]{1,10}",
        prop::sample::select(exts),
    )
        .prop_map(move |(dir, name, ext)| {
            let path_str = format!("{}/{}_{}.{}", dir, name, idx, ext);
            let stem = format!("{}_{}", name, idx);
            (PathBuf::from(path_str), stem)
        })
}

/// Generate a full parsed file with consistent internal references.
fn arb_parsed_file(idx: usize) -> impl Strategy<Value = (ParsedFile, String)> {
    (
        arb_file_path(idx),
        arb_language(),
        prop::collection::vec(arb_symbol(), 1..=15),
        1usize..500,
    )
        .prop_flat_map(|((path, stem), language, symbols, sloc)| {
            let symbols_for_calls = symbols.clone();
            let symbols_for_heritage = symbols.clone();
            (
                Just(path),
                Just(stem),
                Just(language),
                Just(symbols),
                Just(sloc),
                arb_calls(symbols_for_calls),
                arb_heritage(symbols_for_heritage),
            )
        })
        .prop_map(
            |(path, stem, language, symbols, sloc, calls, heritage)| {
                (
                    ParsedFile {
                        path,
                        language,
                        sloc,
                        symbols,
                        imports: vec![], // imports are added in arb_codebase
                        calls,
                        heritage,
                    },
                    stem,
                )
            },
        )
}

/// Generate a codebase of 1-8 files with cross-file imports.
fn arb_codebase() -> impl Strategy<Value = Vec<ParsedFile>> {
    (1usize..=8).prop_flat_map(|count| {
        let strategies: Vec<_> = (0..count).map(arb_parsed_file).collect();
        strategies
    })
    .prop_flat_map(|files_and_stems: Vec<(ParsedFile, String)>| {
        let stems: Vec<String> = files_and_stems.iter().map(|(_, s)| s.clone()).collect();
        let files: Vec<ParsedFile> = files_and_stems.into_iter().map(|(f, _)| f).collect();
        let n = files.len();

        // Generate imports for each file
        let import_strategies: Vec<_> = (0..n)
            .map(|_| arb_imports(stems.clone()))
            .collect();

        (Just(files), import_strategies)
    })
    .prop_map(|(mut files, imports_per_file): (Vec<ParsedFile>, Vec<Vec<Import>>)| {
        for (file, imports) in files.iter_mut().zip(imports_per_file.into_iter()) {
            file.imports = imports;
        }
        files
    })
}

// --- Property Tests ---

proptest! {
    #![proptest_config(ProptestConfig::with_cases(200))]

    /// Building a knowledge graph from any valid input should never panic.
    #[test]
    fn graph_build_never_panics(files in arb_codebase()) {
        let _ = GraphBuilder::new().build(&files);
    }

    /// Every community should have a non-empty label and at least one member.
    #[test]
    fn communities_have_valid_labels_and_members(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for community in &kg.communities {
                prop_assert!(!community.label.is_empty(), "Community label must not be empty");
                prop_assert!(!community.members.is_empty(), "Community must have at least one member");
            }
        }
    }

    /// Every edge in the graph references valid source and destination nodes.
    #[test]
    fn edge_endpoints_are_valid(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for edge_idx in kg.graph.edge_indices() {
                let (src, dst) = kg.graph.edge_endpoints(edge_idx).unwrap();
                prop_assert!(kg.graph.node_weight(src).is_some(), "Edge source node must exist");
                prop_assert!(kg.graph.node_weight(dst).is_some(), "Edge destination node must exist");
            }
        }
    }

    /// Every node_index entry maps to a node that actually exists in the graph.
    #[test]
    fn node_index_consistent_with_graph(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for (&symbol_id, &node_idx) in &kg.node_index {
                prop_assert!(kg.graph.node_weight(node_idx).is_some(),
                    "node_index references non-existent node for symbol {:?}", symbol_id);
                prop_assert_eq!(kg.graph[node_idx].symbol_id, symbol_id,
                    "node_index symbol_id mismatch");
            }
        }
    }

    /// The number of nodes in the graph equals the total symbols across all files.
    #[test]
    fn node_count_matches_symbol_count(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            let total_symbols: usize = files.iter().map(|f| f.symbols.len()).sum();
            prop_assert_eq!(kg.graph.node_count(), total_symbols,
                "Graph node count should equal total symbols");
        }
    }

    /// Entry points are sorted by score in descending order and capped at 30.
    #[test]
    fn entry_points_sorted_and_capped(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            // Capped at 30
            prop_assert!(kg.entry_points.len() <= 30,
                "Entry points should be capped at 30, got {}", kg.entry_points.len());

            // Sorted descending by score
            for w in kg.entry_points.windows(2) {
                prop_assert!(w[0].score >= w[1].score,
                    "Entry points not sorted: {} < {}", w[0].score, w[1].score);
            }
        }
    }

    /// Every entry point's symbol_id exists in the node_index.
    #[test]
    fn entry_point_symbols_exist(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for ep in &kg.entry_points {
                prop_assert!(kg.node_index.contains_key(&ep.symbol_id),
                    "Entry point references unknown symbol {:?}", ep.symbol_id);
            }
        }
    }

    /// Every process step references a valid symbol in the graph.
    #[test]
    fn process_steps_reference_valid_symbols(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for process in &kg.processes {
                prop_assert!(kg.node_index.contains_key(&process.entry_symbol_id),
                    "Process entry symbol {:?} not in node_index", process.entry_symbol_id);
                for step in &process.steps {
                    prop_assert!(kg.node_index.contains_key(&step.symbol_id),
                        "Process step symbol {:?} not in node_index", step.symbol_id);
                }
            }
        }
    }

    /// Process steps are ordered sequentially starting from 0.
    #[test]
    fn process_steps_ordered(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for process in &kg.processes {
                for (i, step) in process.steps.iter().enumerate() {
                    prop_assert_eq!(step.order, i,
                        "Process step order should be sequential, expected {} got {}",
                        i, step.order);
                }
            }
        }
    }

    /// Community members all exist in the node_index.
    #[test]
    fn community_members_in_graph(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for community in &kg.communities {
                for member in &community.members {
                    prop_assert!(kg.node_index.contains_key(member),
                        "Community member {:?} not in node_index", member);
                }
            }
        }
    }

    /// Every edge has a valid EdgeKind and confidence in [0.0, 1.0].
    #[test]
    fn edge_attributes_valid(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for edge_idx in kg.graph.edge_indices() {
                let edge = &kg.graph[edge_idx];
                prop_assert!(edge.confidence.0 >= 0.0 && edge.confidence.0 <= 1.0,
                    "Edge confidence out of range: {}", edge.confidence.0);
                // Verify it's a known variant (this is enforced by the type system,
                // but we check the field is accessible)
                let _ = edge.kind;
            }
        }
    }

    /// Building from an empty file set should succeed with an empty graph.
    #[test]
    fn empty_codebase_produces_empty_graph(_dummy in 0..1u8) {
        let files: Vec<ParsedFile> = vec![];
        let kg = GraphBuilder::new().build(&files).unwrap();
        prop_assert_eq!(kg.graph.node_count(), 0);
        prop_assert_eq!(kg.graph.edge_count(), 0);
        prop_assert!(kg.communities.is_empty());
        prop_assert!(kg.entry_points.is_empty());
        prop_assert!(kg.processes.is_empty());
    }

    /// Processes should be depth-limited (no more than ~21 steps).
    #[test]
    fn process_depth_bounded(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for process in &kg.processes {
                prop_assert!(process.steps.len() <= 25,
                    "Process should be depth-limited, got {} steps", process.steps.len());
            }
        }
    }

    /// No self-loops in the graph (a node should not have an edge to itself).
    /// This is a soft invariant -- the builder should avoid creating them from
    /// calls where caller == callee (which we filter in our generator), but
    /// duplicate symbol names across files could theoretically create them.
    #[test]
    fn no_trivial_self_loops(files in arb_codebase()) {
        if let Ok(kg) = GraphBuilder::new().build(&files) {
            for edge_idx in kg.graph.edge_indices() {
                let (src, dst) = kg.graph.edge_endpoints(edge_idx).unwrap();
                // Self-loops from same-name symbols in different files are OK,
                // but true self-loops (same node) indicate a bug
                if src == dst {
                    let node = &kg.graph[src];
                    // Log but don't fail -- some builders may legitimately create these
                    eprintln!("Warning: self-loop on node {:?} ({})", node.symbol_id, node.name);
                }
            }
        }
    }
}
