use std::collections::HashMap;

use codeilus_core::ids::SymbolId;
use codeilus_core::types::EdgeKind;
use codeilus_graph::KnowledgeGraph;
use petgraph::visit::EdgeRef;

/// Compute fan-in and fan-out for each symbol in the graph.
///
/// Fan-in = number of incoming CALLS edges.
/// Fan-out = number of outgoing CALLS edges.
pub fn compute_fan(graph: &KnowledgeGraph) -> HashMap<SymbolId, (usize, usize)> {
    let mut fan: HashMap<SymbolId, (usize, usize)> = HashMap::new();

    // Initialize all nodes
    for idx in graph.graph.node_indices() {
        let symbol_id = graph.graph[idx].symbol_id;
        fan.entry(symbol_id).or_insert((0, 0));
    }

    // Count edges
    for edge in graph.graph.edge_references() {
        if edge.weight().kind == EdgeKind::Calls {
            let source_id = graph.graph[edge.source()].symbol_id;
            let target_id = graph.graph[edge.target()].symbol_id;

            // fan_out for source
            fan.entry(source_id).or_insert((0, 0)).1 += 1;
            // fan_in for target
            fan.entry(target_id).or_insert((0, 0)).0 += 1;
        }
    }

    fan
}
