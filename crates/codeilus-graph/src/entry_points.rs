use std::collections::HashMap;

use codeilus_core::ids::SymbolId;
use petgraph::graph::DiGraph;

use crate::types::{EntryPoint, GraphEdge, GraphNode};

/// Score and rank entry points in the graph.
pub fn score_entry_points(graph: &DiGraph<GraphNode, GraphEdge>) -> Vec<EntryPoint> {
    // Compute fan-in (incoming edges) for each node
    let mut fan_in: HashMap<SymbolId, usize> = HashMap::new();
    for edge in graph.edge_indices() {
        if let Some((_, target)) = graph.edge_endpoints(edge) {
            let target_id = graph[target].symbol_id;
            *fan_in.entry(target_id).or_default() += 1;
        }
    }

    let mut entry_points: Vec<EntryPoint> = graph
        .node_indices()
        .map(|idx| {
            let node = &graph[idx];
            let mut score = 0.0;
            let mut reasons = Vec::new();

            let name_lower = node.name.to_lowercase();

            // main function → +1.0
            if name_lower == "main" {
                score += 1.0;
                reasons.push("main function");
            }

            // index / mod / __init__ files → +0.5
            if name_lower == "index"
                || name_lower == "mod"
                || name_lower == "__init__"
            {
                score += 0.5;
                reasons.push("module entry");
            }

            // Handler/route patterns → +0.7
            if name_lower.contains("handle")
                || name_lower.contains("handler")
                || name_lower.contains("route")
                || name_lower.contains("endpoint")
            {
                score += 0.7;
                reasons.push("handler/route pattern");
            }

            // CLI patterns → +0.6
            if name_lower.contains("cli")
                || name_lower.contains("cmd")
                || name_lower.contains("command")
            {
                score += 0.6;
                reasons.push("CLI pattern");
            }

            let in_count = fan_in.get(&node.symbol_id).copied().unwrap_or(0);
            let out_count = graph
                .neighbors(idx)
                .count();

            // High fan-in (utility function) → -0.5
            if in_count > 5 {
                score -= 0.5;
                reasons.push("high fan-in (utility)");
            }

            // Zero callers but calls others → +0.5 (true entry point)
            if in_count == 0 && out_count > 0 {
                score += 0.5;
                reasons.push("zero callers, calls others");
            } else if in_count == 0 {
                score += 0.2;
                reasons.push("zero callers");
            }

            EntryPoint {
                symbol_id: node.symbol_id,
                score,
                reason: reasons.join(", "),
            }
        })
        .filter(|ep| ep.score >= 0.5)
        .collect();

    entry_points.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    entry_points.truncate(30);
    entry_points
}
