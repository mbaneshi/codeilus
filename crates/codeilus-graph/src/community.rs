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

                // Standard Louvain modularity gain: ΔQ = ki_in/m - σ_tot·ki/(2m²)
                // where m = total edge weight (sum of all edges)
                let sigma_tot = community_total_degree(&assignments, &degrees, community);
                let delta_q = ki_in / (2.0 * m) - (sigma_tot * ki) / (4.0 * m * m);

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

    // Collect all isolated nodes (communities with 1 member that has no edges) into one "misc" community
    let mut isolated_members: Vec<SymbolId> = Vec::new();
    communities.retain(|c| {
        if c.members.len() == 1 {
            // Check if the single member has any edges
            let sid = c.members[0];
            let has_edges = graph.node_indices().any(|idx| {
                graph[idx].symbol_id == sid
                    && graph.neighbors_undirected(idx).next().is_some()
            });
            if !has_edges {
                isolated_members.push(sid);
                return false;
            }
        }
        true
    });
    if !isolated_members.is_empty() {
        communities.push(Community {
            id: CommunityId(0),
            label: "misc".to_string(),
            members: isolated_members,
            cohesion: 0.0,
        });
    }

    // Merge tiny communities (<=3 members) into nearest connected neighbor's community
    merge_small_communities(graph, &mut communities, 3);

    // Second pass: if still >15 communities, merge communities with <=5 members
    if communities.len() > 15 {
        merge_small_communities(graph, &mut communities, 5);
    }

    // Third pass: if still >15, aggressively merge <=10 members
    if communities.len() > 15 {
        merge_small_communities(graph, &mut communities, 10);
    }

    // Fourth pass: keep merging smallest until <=15 communities
    while communities.len() > 15 {
        let prev_len = communities.len();
        let min_size = communities.last().map(|c| c.members.len()).unwrap_or(0);
        merge_small_communities(graph, &mut communities, min_size);
        if communities.len() >= prev_len {
            break; // no progress, stop
        }
    }

    // Re-assign sequential IDs
    for (i, c) in communities.iter_mut().enumerate() {
        c.id = CommunityId(i as i64 + 1);
    }

    communities
}

/// Merge communities with <= `threshold` members into their most-connected neighbor community.
fn merge_small_communities(
    graph: &DiGraph<GraphNode, GraphEdge>,
    communities: &mut Vec<Community>,
    threshold: usize,
) {
    if communities.len() <= 1 {
        return;
    }

    // Build reverse map: symbol_id -> node_index
    let node_index_map: HashMap<SymbolId, NodeIndex> = graph
        .node_indices()
        .map(|idx| (graph[idx].symbol_id, idx))
        .collect();

    // Build a map: symbol_id -> community index
    let mut sym_to_comm: HashMap<SymbolId, usize> = HashMap::new();
    for (ci, c) in communities.iter().enumerate() {
        for &sid in &c.members {
            sym_to_comm.insert(sid, ci);
        }
    }

    // Find small communities and merge them
    let mut merges: Vec<(usize, usize)> = Vec::new(); // (from_idx, to_idx)
    for (ci, c) in communities.iter().enumerate() {
        if c.members.len() > threshold {
            continue;
        }
        // Find the most-connected neighbor community
        let mut neighbor_comm_edges: HashMap<usize, usize> = HashMap::new();
        for &sid in &c.members {
            if let Some(&idx) = node_index_map.get(&sid) {
                for neighbor in graph.neighbors_undirected(idx) {
                    let neighbor_sid = graph[neighbor].symbol_id;
                    if let Some(&nci) = sym_to_comm.get(&neighbor_sid) {
                        if nci != ci {
                            *neighbor_comm_edges.entry(nci).or_default() += 1;
                        }
                    }
                }
            }
        }
        let target_comm = neighbor_comm_edges
            .into_iter()
            .max_by_key(|&(_, count)| count)
            .map(|(nci, _)| nci);
        // Only merge if we found a viable larger neighbor
        if let Some(target) = target_comm {
            if target != ci {
                merges.push((ci, target));
            }
        }
    }

    // Apply merges (collect members, then move)
    for (from, to) in merges {
        let members: Vec<SymbolId> = communities[from].members.clone();
        communities[to].members.extend(members);
        communities[from].members.clear();
    }

    // Remove empty communities and re-sort
    communities.retain(|c| !c.members.is_empty());
    communities.sort_by(|a, b| b.members.len().cmp(&a.members.len()));
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
