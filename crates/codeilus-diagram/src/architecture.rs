//! Architecture diagram: communities → Mermaid subgraphs.

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use petgraph::graph::NodeIndex;
use std::collections::{HashMap, HashSet};
use tracing::debug;

use crate::mermaid::{escape_label, sanitize_node_id};

/// Maximum number of nodes to include in the architecture diagram.
const MAX_NODES: usize = 50;

/// Generate an architecture Mermaid diagram from the knowledge graph.
pub fn generate(graph: &KnowledgeGraph) -> CodeilusResult<String> {
    let mut output = String::from("graph TD\n");

    // Collect all node indices and their fan-in counts for ranking
    let mut fan_in: HashMap<NodeIndex, usize> = HashMap::new();
    for edge in graph.graph.edge_indices() {
        if let Some((_src, tgt)) = graph.graph.edge_endpoints(edge) {
            *fan_in.entry(tgt).or_default() += 1;
        }
    }

    // Sort nodes by fan-in descending, take top MAX_NODES
    let mut all_nodes: Vec<NodeIndex> =
        graph.graph.node_indices().collect();
    all_nodes.sort_by(|a, b| {
        fan_in
            .get(b)
            .unwrap_or(&0)
            .cmp(fan_in.get(a).unwrap_or(&0))
    });
    let included_nodes: HashSet<NodeIndex> =
        all_nodes.iter().take(MAX_NODES).copied().collect();

    // Build community → node index mapping
    let mut community_nodes: HashMap<String, Vec<NodeIndex>> = HashMap::new();
    let mut uncategorized: Vec<NodeIndex> = Vec::new();

    for &idx in &included_nodes {
        let node = &graph.graph[idx];
        if let Some(cid) = &node.community_id {
            // Find the community label
            let label = graph
                .communities
                .iter()
                .find(|c| c.id == *cid)
                .map(|c| c.label.clone())
                .unwrap_or_else(|| format!("Community {}", cid.0));
            community_nodes
                .entry(label)
                .or_default()
                .push(idx);
        } else {
            uncategorized.push(idx);
        }
    }

    // Sort communities by name for deterministic output
    let mut sorted_communities: Vec<_> = community_nodes.into_iter().collect();
    sorted_communities.sort_by(|a, b| a.0.cmp(&b.0));

    // Emit subgraphs
    for (i, (label, nodes)) in sorted_communities.iter().enumerate() {
        let subgraph_id = format!("C{}", i);
        let escaped_label = escape_label(label);
        output.push_str(&format!(
            "    subgraph {}[\"{}\"]\n",
            subgraph_id, escaped_label
        ));
        for &idx in nodes {
            let node = &graph.graph[idx];
            let node_id = sanitize_node_id(&format!("n{}_{}", idx.index(), node.name));
            let label = escape_label(&format!("{} ({})", node.name, node.kind));
            output.push_str(&format!("        {}[\"{}\"]\n", node_id, label));
        }
        output.push_str("    end\n");
    }

    // Emit uncategorized nodes
    for &idx in &uncategorized {
        let node = &graph.graph[idx];
        let node_id = sanitize_node_id(&format!("n{}_{}", idx.index(), node.name));
        let label = escape_label(&format!("{} ({})", node.name, node.kind));
        output.push_str(&format!("    {}[\"{}\"]\n", node_id, label));
    }

    // Build node_index → mermaid_id mapping for edges
    let mut node_id_map: HashMap<NodeIndex, String> = HashMap::new();
    for &idx in &included_nodes {
        let node = &graph.graph[idx];
        node_id_map.insert(
            idx,
            sanitize_node_id(&format!("n{}_{}", idx.index(), node.name)),
        );
    }

    // Emit inter-community edges as dashed arrows
    for edge_idx in graph.graph.edge_indices() {
        if let Some((src, tgt)) = graph.graph.edge_endpoints(edge_idx) {
            if !included_nodes.contains(&src) || !included_nodes.contains(&tgt) {
                continue;
            }
            let src_node = &graph.graph[src];
            let tgt_node = &graph.graph[tgt];
            // Only draw inter-community edges (or edges involving uncategorized nodes)
            let same_community = match (&src_node.community_id, &tgt_node.community_id) {
                (Some(a), Some(b)) => a == b,
                _ => false,
            };
            if !same_community {
                let src_id = &node_id_map[&src];
                let tgt_id = &node_id_map[&tgt];
                output.push_str(&format!("    {} -.-> {}\n", src_id, tgt_id));
            }
        }
    }

    debug!(
        nodes = included_nodes.len(),
        "generated architecture diagram"
    );
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeilus_core::ids::{CommunityId, FileId, SymbolId};
    use codeilus_core::types::{Confidence, EdgeKind};
    use codeilus_graph::{Community, GraphEdge, GraphNode, KnowledgeGraph};
    use petgraph::graph::DiGraph;
    use std::collections::HashMap;

    fn make_graph_with_communities(
        num_communities: usize,
        nodes_per_community: usize,
    ) -> KnowledgeGraph {
        let mut graph = DiGraph::new();
        let mut node_index = HashMap::new();
        let mut communities = Vec::new();

        for c in 0..num_communities {
            let cid = CommunityId(c as i64);
            let mut members = Vec::new();
            for n in 0..nodes_per_community {
                let sid = SymbolId((c * nodes_per_community + n) as i64);
                let idx = graph.add_node(GraphNode {
                    symbol_id: sid,
                    file_id: FileId(0),
                    name: format!("func_{}_{}", c, n),
                    kind: "fn".to_string(),
                    community_id: Some(cid),
                });
                node_index.insert(sid, idx);
                members.push(sid);
            }
            communities.push(Community {
                id: cid,
                label: format!("Module {}", c),
                members,
                cohesion: 0.8,
            });
        }

        KnowledgeGraph {
            graph,
            node_index,
            communities,
            processes: vec![],
            entry_points: vec![],
        }
    }

    #[test]
    fn architecture_two_communities() {
        let kg = make_graph_with_communities(2, 3);
        let result = generate(&kg).unwrap();
        assert!(result.starts_with("graph TD"));
        assert!(result.contains("subgraph"));
        assert!(result.contains("Module 0"));
        assert!(result.contains("Module 1"));
        // Two subgraphs and two ends
        assert_eq!(result.matches("subgraph").count(), 2);
        assert_eq!(result.matches("end").count(), 2);
    }

    #[test]
    fn architecture_inter_community_edges() {
        let mut kg = make_graph_with_communities(2, 2);
        // Add edge from community 0 node to community 1 node
        let src = *kg.node_index.get(&SymbolId(0)).unwrap();
        let tgt = *kg.node_index.get(&SymbolId(2)).unwrap();
        kg.graph.add_edge(
            src,
            tgt,
            GraphEdge {
                kind: EdgeKind::Calls,
                confidence: Confidence::high(),
            },
        );
        let result = generate(&kg).unwrap();
        assert!(result.contains("-.->"), "Should have dashed inter-community edge");
    }

    #[test]
    fn architecture_label_escaping() {
        let mut graph = DiGraph::new();
        let mut node_index = HashMap::new();
        let sid = SymbolId(0);
        let idx = graph.add_node(GraphNode {
            symbol_id: sid,
            file_id: FileId(0),
            name: "handle(\"request\")".to_string(),
            kind: "fn".to_string(),
            community_id: Some(CommunityId(0)),
        });
        node_index.insert(sid, idx);
        let communities = vec![Community {
            id: CommunityId(0),
            label: "HTTP \"Layer\"".to_string(),
            members: vec![sid],
            cohesion: 0.9,
        }];
        let kg = KnowledgeGraph {
            graph,
            node_index,
            communities,
            processes: vec![],
            entry_points: vec![],
        };
        let result = generate(&kg).unwrap();
        // Escaped quotes should not appear as raw "
        assert!(!result.contains("handle(\"request\")"));
        assert!(result.contains("#quot;") || result.contains("#lpar;"));
    }

    #[test]
    fn architecture_node_limit() {
        // 100 nodes across 10 communities → only 50 in output
        let kg = make_graph_with_communities(10, 10);
        let result = generate(&kg).unwrap();
        // Count node definitions (lines with ["..."])
        let node_lines = result
            .lines()
            .filter(|l| l.contains("[\"") && !l.contains("subgraph"))
            .count();
        assert!(
            node_lines <= MAX_NODES,
            "Expected at most {} nodes, got {}",
            MAX_NODES,
            node_lines
        );
    }
}
