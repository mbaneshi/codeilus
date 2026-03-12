use std::collections::{HashSet, VecDeque};

use codeilus_core::ids::SymbolId;
use codeilus_core::types::EdgeKind;
use petgraph::graph::{DiGraph, NodeIndex};
use petgraph::visit::EdgeRef;

use crate::types::{EntryPoint, GraphEdge, GraphNode, Process, ProcessStep};

const MAX_BFS_DEPTH: usize = 20;

/// Detect execution flows via BFS from entry points through CALLS edges.
pub fn detect_processes(
    graph: &DiGraph<GraphNode, GraphEdge>,
    entry_points: &[EntryPoint],
    node_index: &std::collections::HashMap<SymbolId, NodeIndex>,
) -> Vec<Process> {
    let mut processes = Vec::new();

    for ep in entry_points {
        let start_idx = match node_index.get(&ep.symbol_id) {
            Some(idx) => *idx,
            None => continue,
        };

        let steps = bfs_calls(graph, start_idx);
        if steps.is_empty() {
            continue;
        }

        let name = format!("{}_flow", graph[start_idx].name);
        processes.push(Process {
            name,
            entry_symbol_id: ep.symbol_id,
            steps,
        });
    }

    processes
}

/// BFS from a start node following only CALLS edges.
fn bfs_calls(graph: &DiGraph<GraphNode, GraphEdge>, start: NodeIndex) -> Vec<ProcessStep> {
    let mut visited = HashSet::new();
    let mut queue: VecDeque<(NodeIndex, usize)> = VecDeque::new();
    let mut steps = Vec::new();

    visited.insert(start);
    queue.push_back((start, 0));

    while let Some((node_idx, depth)) = queue.pop_front() {
        if depth > MAX_BFS_DEPTH {
            break;
        }

        let node = &graph[node_idx];
        steps.push(ProcessStep {
            order: steps.len(),
            symbol_id: node.symbol_id,
            description: format!("Step {}: {}", steps.len(), node.name),
        });

        // Follow CALLS edges only
        for edge in graph.edges(node_idx) {
            if edge.weight().kind == EdgeKind::Calls && !visited.contains(&edge.target()) {
                visited.insert(edge.target());
                queue.push_back((edge.target(), depth + 1));
            }
        }
    }

    steps
}
