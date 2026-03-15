use std::collections::HashMap;

use codeilus_core::ids::{FileId, SymbolId};
use codeilus_core::types::{Confidence, EdgeKind};
use codeilus_core::CodeilusResult;
use codeilus_parse::ParsedFile;
use petgraph::graph::{DiGraph, NodeIndex};

use crate::types::{Community, GraphEdge, GraphNode, KnowledgeGraph};
use crate::{call_graph, community, dep_graph, entry_points, heritage, process};

/// Orchestrates building the full knowledge graph from parsed files.
pub struct GraphBuilder;

impl Default for GraphBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl GraphBuilder {
    pub fn new() -> Self {
        Self
    }

    /// Build the full knowledge graph from parsed files.
    ///
    /// 1. Build symbol index (name → SymbolId mapping)
    /// 2. Construct call graph edges
    /// 3. Construct dependency graph edges
    /// 4. Construct heritage edges
    /// 5. Run Louvain community detection
    /// 6. Score entry points
    /// 7. Detect execution flows
    pub fn build(&self, parsed_files: &[ParsedFile]) -> CodeilusResult<KnowledgeGraph> {
        // Step 1: Build indexes
        let mut graph = DiGraph::new();
        let mut node_index = HashMap::new();
        let mut symbol_index: HashMap<String, Vec<SymbolId>> = HashMap::new();
        let mut name_to_id: HashMap<(String, String), SymbolId> = HashMap::new();
        let mut file_index: HashMap<String, FileId> = HashMap::new();

        let mut next_symbol_id: i64 = 1;
        let mut next_file_id: i64 = 1;

        for pf in parsed_files {
            let file_path = pf.path.to_string_lossy().to_string();
            let file_id = FileId(next_file_id);
            next_file_id += 1;
            file_index.insert(file_path.clone(), file_id);

            for sym in &pf.symbols {
                let symbol_id = SymbolId(next_symbol_id);
                next_symbol_id += 1;

                let node = GraphNode {
                    symbol_id,
                    file_id,
                    name: sym.name.clone(),
                    kind: format!("{:?}", sym.kind),
                    community_id: None,
                };

                let idx = graph.add_node(node);
                node_index.insert(symbol_id, idx);

                symbol_index
                    .entry(sym.name.clone())
                    .or_default()
                    .push(symbol_id);
                name_to_id.insert((sym.name.clone(), file_path.clone()), symbol_id);
            }
        }

        // Step 2: Call graph edges
        let call_edges = call_graph::build_call_edges(parsed_files, &symbol_index, &name_to_id);
        for (caller, callee, confidence) in &call_edges {
            if let (Some(&caller_idx), Some(&callee_idx)) =
                (node_index.get(caller), node_index.get(callee))
            {
                graph.add_edge(
                    caller_idx,
                    callee_idx,
                    GraphEdge {
                        kind: EdgeKind::Calls,
                        confidence: *confidence,
                    },
                );
            }
        }

        // Step 3: Dependency graph edges — promote file-level deps to symbol-level Imports edges
        let dep_edges = dep_graph::build_dep_edges(parsed_files, &file_index);
        for (source_fid, target_fid) in &dep_edges {
            let src_sym = graph
                .node_indices()
                .find(|&i| graph[i].file_id == *source_fid);
            let tgt_sym = graph
                .node_indices()
                .find(|&i| graph[i].file_id == *target_fid);
            if let (Some(src), Some(tgt)) = (src_sym, tgt_sym) {
                graph.add_edge(
                    src,
                    tgt,
                    GraphEdge {
                        kind: EdgeKind::Imports,
                        confidence: Confidence(0.6),
                    },
                );
            }
        }

        // Step 4: Heritage edges
        let heritage_edges = heritage::build_heritage_edges(parsed_files, &symbol_index);
        for (child, parent, kind) in &heritage_edges {
            if let (Some(&child_idx), Some(&parent_idx)) =
                (node_index.get(child), node_index.get(parent))
            {
                graph.add_edge(
                    child_idx,
                    parent_idx,
                    GraphEdge {
                        kind: *kind,
                        confidence: Confidence::certain(),
                    },
                );
            }
        }

        // Step 5: Community detection
        let mut communities = community::detect_communities(&graph);

        // Assign community IDs back to graph nodes
        for community in &communities {
            for &member_id in &community.members {
                if let Some(&idx) = node_index.get(&member_id) {
                    graph[idx].community_id = Some(community.id);
                }
            }
        }

        // Step 6: Entry point scoring
        let entry_pts = entry_points::score_entry_points(&graph);

        // Step 7: Process detection
        let processes = process::detect_processes(&graph, &entry_pts, &node_index);

        // Label communities semantically using TF-IDF on symbol names
        label_communities_semantic(&graph, &mut communities, &node_index);

        Ok(KnowledgeGraph {
            graph,
            node_index,
            communities,
            processes,
            entry_points: entry_pts,
        })
    }
}

/// Label communities semantically using TF-IDF on symbol names within each community.
fn label_communities_semantic(
    graph: &DiGraph<GraphNode, GraphEdge>,
    communities: &mut [Community],
    node_index: &HashMap<SymbolId, NodeIndex>,
) {
    // Collect symbol names per community for TF-IDF
    let community_names: Vec<Vec<String>> = communities
        .iter()
        .map(|c| {
            c.members
                .iter()
                .filter_map(|sid| node_index.get(sid).map(|&idx| graph[idx].name.clone()))
                .collect()
        })
        .collect();

    let n_communities = community_names.len();
    if n_communities == 0 {
        return;
    }

    // Tokenize all names per community
    let community_tokens: Vec<Vec<String>> = community_names
        .iter()
        .map(|names| names.iter().flat_map(|n| tokenize_name(n)).collect())
        .collect();

    // Document frequency: how many communities contain each token
    let mut df: HashMap<String, usize> = HashMap::new();
    for tokens in &community_tokens {
        let unique: std::collections::HashSet<&String> = tokens.iter().collect();
        for token in unique {
            *df.entry(token.clone()).or_default() += 1;
        }
    }

    // Generic terms to avoid as labels
    let stop_words: std::collections::HashSet<&str> = [
        "new", "get", "set", "run", "self", "impl", "pub", "fn", "let", "mut",
        "str", "string", "vec", "option", "result", "default", "from", "into",
        "test", "tests", "mod", "use", "type", "id", "name", "data", "value",
    ]
    .iter()
    .copied()
    .collect();

    // Compute TF-IDF and pick best label per community
    for (i, community) in communities.iter_mut().enumerate() {
        let tokens = &community_tokens[i];
        if tokens.is_empty() {
            community.label = format!("group_{}", i + 1);
            continue;
        }

        // Term frequency
        let mut tf: HashMap<String, usize> = HashMap::new();
        for token in tokens {
            *tf.entry(token.clone()).or_default() += 1;
        }

        let total = tokens.len() as f64;
        let mut scores: Vec<(String, f64)> = tf
            .iter()
            .filter(|(term, _)| !stop_words.contains(term.as_str()))
            .map(|(term, &count)| {
                let tf_val = count as f64 / total;
                let df_val = df.get(term).copied().unwrap_or(1) as f64;
                let idf_val = (n_communities as f64 / df_val).ln() + 1.0;
                (term.clone(), tf_val * idf_val)
            })
            .collect();

        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

        // Take top 2 keywords, join with underscore
        let label = match scores.len() {
            0 => format!("group_{}", i + 1),
            1 => scores[0].0.clone(),
            _ => format!("{}_{}", scores[0].0, scores[1].0),
        };

        community.label = label;
    }

    // Deduplicate labels: if two communities have the same label, append a suffix
    let mut label_counts: HashMap<String, usize> = HashMap::new();
    for community in communities.iter_mut() {
        let count = label_counts.entry(community.label.clone()).or_default();
        if *count > 0 {
            community.label = format!("{}_{}", community.label, *count + 1);
        }
        *count += 1;
    }
}

/// Tokenize a symbol name into words (handles camelCase, PascalCase, snake_case).
fn tokenize_name(name: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let chars: Vec<char> = name.chars().collect();

    for i in 0..chars.len() {
        let ch = chars[i];
        if ch == '_' || ch == '-' || ch == '.' || ch == ':' {
            if !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
        } else if ch.is_uppercase() {
            let prev_lower = i > 0 && chars[i - 1].is_lowercase();
            let next_lower = i + 1 < chars.len() && chars[i + 1].is_lowercase();
            let in_upper_run = i > 0 && chars[i - 1].is_uppercase();
            if (prev_lower || (in_upper_run && next_lower)) && !current.is_empty() {
                tokens.push(current.to_lowercase());
                current.clear();
            }
            current.push(ch);
        } else {
            current.push(ch);
        }
    }
    if !current.is_empty() {
        tokens.push(current.to_lowercase());
    }

    tokens.into_iter().filter(|t| t.len() >= 3).collect()
}
