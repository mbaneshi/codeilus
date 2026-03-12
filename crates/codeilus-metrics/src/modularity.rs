use codeilus_core::ids::CommunityId;
use codeilus_graph::KnowledgeGraph;
use std::collections::{HashMap, HashSet};

/// Compute Louvain modularity Q-score from the knowledge graph.
///
/// Uses undirected projection: Q = (1/2m) * Σ [A_ij - (k_i * k_j)/(2m)] * δ(c_i, c_j)
pub fn compute_modularity(graph: &KnowledgeGraph) -> (f64, HashMap<CommunityId, f64>) {
    let g = &graph.graph;

    if g.edge_count() == 0 {
        let per_community: HashMap<CommunityId, f64> = graph
            .communities
            .iter()
            .map(|c| (c.id, 0.0))
            .collect();
        return (0.0, per_community);
    }

    // Build undirected adjacency: count unique undirected edges
    let mut undirected_edges: HashSet<(usize, usize)> = HashSet::new();
    for edge_idx in g.edge_indices() {
        if let Some((src, tgt)) = g.edge_endpoints(edge_idx) {
            let a = src.index().min(tgt.index());
            let b = src.index().max(tgt.index());
            undirected_edges.insert((a, b));
        }
    }

    let m = undirected_edges.len() as f64;
    let two_m = 2.0 * m;

    // Compute undirected degree for each node
    let mut degrees: HashMap<petgraph::graph::NodeIndex, f64> = HashMap::new();
    for idx in g.node_indices() {
        degrees.insert(idx, 0.0);
    }
    for &(a, b) in &undirected_edges {
        let idx_a = petgraph::graph::NodeIndex::new(a);
        let idx_b = petgraph::graph::NodeIndex::new(b);
        *degrees.entry(idx_a).or_default() += 1.0;
        *degrees.entry(idx_b).or_default() += 1.0;
    }

    // Build undirected adjacency check
    let adj = &undirected_edges;

    let mut global_q = 0.0;
    let mut per_community: HashMap<CommunityId, f64> = HashMap::new();

    for community in &graph.communities {
        let mut community_q = 0.0;

        for &member_i in &community.members {
            for &member_j in &community.members {
                let idx_i = match graph.node_index.get(&member_i) {
                    Some(idx) => *idx,
                    None => continue,
                };
                let idx_j = match graph.node_index.get(&member_j) {
                    Some(idx) => *idx,
                    None => continue,
                };

                let a = idx_i.index().min(idx_j.index());
                let b = idx_i.index().max(idx_j.index());

                // A_ij: undirected adjacency
                let a_ij = if idx_i == idx_j {
                    0.0 // no self-loops in modularity
                } else if adj.contains(&(a, b)) {
                    1.0
                } else {
                    0.0
                };

                let k_i = degrees.get(&idx_i).copied().unwrap_or(0.0);
                let k_j = degrees.get(&idx_j).copied().unwrap_or(0.0);

                community_q += a_ij - (k_i * k_j) / two_m;
            }
        }

        community_q /= two_m;
        per_community.insert(community.id, community_q);
        global_q += community_q;
    }

    (global_q, per_community)
}
