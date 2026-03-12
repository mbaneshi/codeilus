use std::collections::HashMap;

use codeilus_core::ids::CommunityId;
use codeilus_graph::KnowledgeGraph;
use petgraph::visit::EdgeRef;

/// Per-community modularity contribution.
pub struct ModularityResult {
    pub global_q: f64,
    pub per_community: HashMap<CommunityId, f64>,
}

/// Compute Louvain modularity Q-score from community assignments.
///
/// Q = (1/2m) * Σ [A_ij - (k_i * k_j)/(2m)] * δ(c_i, c_j)
pub fn compute_modularity(graph: &KnowledgeGraph) -> ModularityResult {
    let m = graph.graph.edge_count() as f64;

    if m == 0.0 {
        return ModularityResult {
            global_q: 0.0,
            per_community: HashMap::new(),
        };
    }

    // Build community assignment map
    let mut node_community: HashMap<petgraph::graph::NodeIndex, CommunityId> = HashMap::new();
    for idx in graph.graph.node_indices() {
        if let Some(cid) = graph.graph[idx].community_id {
            node_community.insert(idx, cid);
        }
    }

    // Compute degree for each node (total edges, both directions)
    let mut degrees: HashMap<petgraph::graph::NodeIndex, f64> = HashMap::new();
    for edge in graph.graph.edge_references() {
        *degrees.entry(edge.source()).or_default() += 1.0;
        *degrees.entry(edge.target()).or_default() += 1.0;
    }

    let two_m = 2.0 * m;
    let mut global_q = 0.0;
    let mut per_community: HashMap<CommunityId, f64> = HashMap::new();

    // For each edge, compute modularity contribution
    for edge in graph.graph.edge_references() {
        let i = edge.source();
        let j = edge.target();

        let ci = node_community.get(&i);
        let cj = node_community.get(&j);

        if ci.is_some() && ci == cj {
            let ki = degrees.get(&i).copied().unwrap_or(0.0);
            let kj = degrees.get(&j).copied().unwrap_or(0.0);

            let contribution = 1.0 - (ki * kj) / two_m;
            global_q += contribution;

            if let Some(&cid) = ci {
                *per_community.entry(cid).or_default() += contribution;
            }
        }
    }

    global_q /= two_m;

    // Normalize per-community
    for val in per_community.values_mut() {
        *val /= two_m;
    }

    ModularityResult {
        global_q,
        per_community,
    }
}
