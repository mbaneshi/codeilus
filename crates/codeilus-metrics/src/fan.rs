use std::collections::HashMap;

use codeilus_core::ids::SymbolId;
use codeilus_core::types::EdgeKind;
use codeilus_graph::KnowledgeGraph;

/// Compute fan-in and fan-out for each symbol in the graph.
/// Fan-in = incoming CALLS edges, Fan-out = outgoing CALLS edges.
pub fn compute_fan(graph: &KnowledgeGraph) -> HashMap<SymbolId, (usize, usize)> {
    let mut fan: HashMap<SymbolId, (usize, usize)> = HashMap::new();

    // Initialize all nodes
    for idx in graph.graph.node_indices() {
        let node = &graph.graph[idx];
        fan.entry(node.symbol_id).or_insert((0, 0));
    }

    // Count edges
    for edge_idx in graph.graph.edge_indices() {
        let edge = &graph.graph[edge_idx];
        if edge.kind == EdgeKind::Calls {
            if let Some((source, target)) = graph.graph.edge_endpoints(edge_idx) {
                let source_id = graph.graph[source].symbol_id;
                let target_id = graph.graph[target].symbol_id;

                fan.entry(source_id).or_insert((0, 0)).1 += 1; // fan_out
                fan.entry(target_id).or_insert((0, 0)).0 += 1; // fan_in
            }
        }
    }

    fan
}
