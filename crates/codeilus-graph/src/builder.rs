use std::collections::HashMap;

use codeilus_core::ids::{FileId, SymbolId};
use codeilus_core::types::{Confidence, EdgeKind};
use codeilus_core::CodeilusResult;
use codeilus_parse::ParsedFile;
use petgraph::graph::DiGraph;

use crate::types::{GraphEdge, GraphNode, KnowledgeGraph};
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

        // Step 3: Dependency graph edges (file-level)
        let dep_edges = dep_graph::build_dep_edges(parsed_files, &file_index);
        // Note: dep edges are file-level, not added to symbol graph but tracked
        let _ = dep_edges;

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

        // Label communities based on most common symbol kind
        for community in &mut communities {
            if !community.members.is_empty() {
                let first_member = community.members[0];
                if let Some(&idx) = node_index.get(&first_member) {
                    let name = &graph[idx].name;
                    community.label = format!("cluster_{name}");
                }
            }
        }

        Ok(KnowledgeGraph {
            graph,
            node_index,
            communities,
            processes,
            entry_points: entry_pts,
        })
    }
}
