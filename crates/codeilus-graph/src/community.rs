use std::collections::HashMap;

use codeilus_core::ids::{CommunityId, SymbolId};
use petgraph::graph::{DiGraph, NodeIndex};

use crate::types::{Community, GraphEdge, GraphNode};

/// Run Louvain community detection on the graph.
///
/// Algorithm:
/// 1. Each node starts as its own community
/// 2. Move nodes to neighbor's community if modularity increases
/// 3. Repeat until no improvement
pub fn detect_communities(graph: &DiGraph<GraphNode, GraphEdge>) -> Vec<Community> {
    let node_count = graph.node_count();
    if node_count == 0 {
        return Vec::new();
    }

    // Initialize: each node in its own community
    let mut assignments: HashMap<NodeIndex, usize> = HashMap::new();
    for idx in graph.node_indices() {
        assignments.insert(idx, idx.index());
    }

    // Total edge weight (number of edges for unweighted)
    let m = graph.edge_count() as f64;
    if m == 0.0 {
        // No edges: all nodes in one community
        let members: Vec<SymbolId> = graph
            .node_indices()
            .map(|idx| graph[idx].symbol_id)
            .collect();
        return vec![Community {
            id: CommunityId(1),
            label: "community_0".to_string(),
            members,
            cohesion: 1.0,
        }];
    }

    // Precompute degree for each node (in + out for directed graph)
    let degrees: HashMap<NodeIndex, f64> = graph
        .node_indices()
        .map(|idx| {
            let degree = graph.neighbors_undirected(idx).count() as f64;
            (idx, degree)
        })
        .collect();

    // Iterative modularity optimization
    let mut improved = true;
    let max_iterations = 20;
    let mut iteration = 0;

    while improved && iteration < max_iterations {
        improved = false;
        iteration += 1;

        for node_idx in graph.node_indices() {
            let current_community = assignments[&node_idx];

            // Collect neighboring communities
            let mut neighbor_communities: HashMap<usize, f64> = HashMap::new();
            for neighbor in graph.neighbors_undirected(node_idx) {
                let nc = assignments[&neighbor];
                *neighbor_communities.entry(nc).or_default() += 1.0;
            }

            // Find best community
            let ki = degrees[&node_idx];
            let mut best_community = current_community;
            let mut best_delta = 0.0;

            for (&community, &ki_in) in &neighbor_communities {
                if community == current_community {
                    continue;
                }

                // Simplified modularity gain calculation
                let sigma_tot = community_total_degree(&assignments, &degrees, community);
                let delta_q = ki_in / m - (sigma_tot * ki) / (2.0 * m * m);

                if delta_q > best_delta {
                    best_delta = delta_q;
                    best_community = community;
                }
            }

            if best_community != current_community {
                assignments.insert(node_idx, best_community);
                improved = true;
            }
        }
    }

    // Collect communities
    let mut community_members: HashMap<usize, Vec<NodeIndex>> = HashMap::new();
    for (node_idx, community) in &assignments {
        community_members
            .entry(*community)
            .or_default()
            .push(*node_idx);
    }

    let mut communities: Vec<Community> = community_members
        .into_iter()
        .enumerate()
        .map(|(i, (_, nodes))| {
            let members: Vec<SymbolId> = nodes
                .iter()
                .map(|idx| graph[*idx].symbol_id)
                .collect();

            let cohesion = compute_cohesion(graph, &nodes);

            Community {
                id: CommunityId(i as i64 + 1),
                label: format!("community_{i}"),
                members,
                cohesion,
            }
        })
        .collect();

    communities.sort_by(|a, b| b.members.len().cmp(&a.members.len()));

    // Re-assign sequential IDs
    for (i, c) in communities.iter_mut().enumerate() {
        c.id = CommunityId(i as i64 + 1);
    }

    communities
}

/// Sum of degrees of all nodes in a community.
fn community_total_degree(
    assignments: &HashMap<NodeIndex, usize>,
    degrees: &HashMap<NodeIndex, f64>,
    community: usize,
) -> f64 {
    assignments
        .iter()
        .filter(|(_, &c)| c == community)
        .map(|(node, _)| degrees.get(node).copied().unwrap_or(0.0))
        .sum()
}

/// Compute cohesion: ratio of internal edges to total possible edges.
fn compute_cohesion(graph: &DiGraph<GraphNode, GraphEdge>, nodes: &[NodeIndex]) -> f64 {
    if nodes.len() <= 1 {
        return 1.0;
    }

    let node_set: std::collections::HashSet<NodeIndex> = nodes.iter().copied().collect();
    let mut internal_edges = 0;

    for &node in nodes {
        for neighbor in graph.neighbors_undirected(node) {
            if node_set.contains(&neighbor) {
                internal_edges += 1;
            }
        }
    }

    // Each edge counted twice in undirected traversal
    let internal_edges = internal_edges / 2;
    let max_edges = nodes.len() * (nodes.len() - 1) / 2;

    if max_edges == 0 {
        1.0
    } else {
        internal_edges as f64 / max_edges as f64
    }
}
