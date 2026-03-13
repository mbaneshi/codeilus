//! Build context strings from KnowledgeGraph for LLM prompts.

use codeilus_core::ids::{CommunityId, SymbolId};
use codeilus_graph::KnowledgeGraph;
use petgraph::visit::EdgeRef;

use crate::types::ContextFocus;

/// Maximum context size in characters (~8K tokens * 4 chars/token).
const MAX_CONTEXT_CHARS: usize = 32_000;

/// Build a context string from the knowledge graph for a given focus.
pub fn build_context(graph: &KnowledgeGraph, focus: ContextFocus) -> String {
    let raw = match focus {
        ContextFocus::Overview => build_overview(graph),
        ContextFocus::Community(id) => build_community(graph, CommunityId(id)),
        ContextFocus::Symbol(id) => build_symbol(graph, SymbolId(id)),
        ContextFocus::Files(paths) => build_files(graph, &paths),
    };

    // Truncate to budget
    if raw.len() > MAX_CONTEXT_CHARS {
        let mut truncated = raw[..MAX_CONTEXT_CHARS - 50].to_string();
        truncated.push_str("\n\n... (context truncated for token budget)");
        truncated
    } else {
        raw
    }
}

fn build_overview(graph: &KnowledgeGraph) -> String {
    let mut ctx = String::from("# Repository Overview\n\n");

    // Node/edge counts
    let node_count = graph.graph.node_count();
    let edge_count = graph.graph.edge_count();
    ctx.push_str(&format!(
        "- **Symbols:** {}\n- **Relationships:** {}\n",
        node_count, edge_count
    ));

    // Communities
    ctx.push_str(&format!(
        "- **Communities:** {}\n\n",
        graph.communities.len()
    ));

    // Community listing
    if !graph.communities.is_empty() {
        ctx.push_str("## Communities\n\n");
        for (i, community) in graph.communities.iter().enumerate() {
            ctx.push_str(&format!(
                "{}. **{}** — {} members (cohesion: {:.2})\n",
                i + 1,
                community.label,
                community.members.len(),
                community.cohesion
            ));

            // List top members (by name), limit to 10
            let member_names: Vec<String> = community
                .members
                .iter()
                .filter_map(|sid| {
                    graph
                        .node_index
                        .get(sid)
                        .map(|&idx| graph.graph[idx].name.clone())
                })
                .take(10)
                .collect();

            if !member_names.is_empty() {
                ctx.push_str(&format!("   Members: {}", member_names.join(", ")));
                if community.members.len() > 10 {
                    ctx.push_str(&format!(" ... and {} more", community.members.len() - 10));
                }
                ctx.push('\n');
            }
        }
        ctx.push('\n');
    }

    // Entry points
    if !graph.entry_points.is_empty() {
        ctx.push_str("## Entry Points\n\n");
        let limit = 10.min(graph.entry_points.len());
        for ep in &graph.entry_points[..limit] {
            let name = graph
                .node_index
                .get(&ep.symbol_id)
                .map(|&idx| graph.graph[idx].name.as_str())
                .unwrap_or("unknown");
            ctx.push_str(&format!(
                "- **{}** (score: {:.2}) — {}\n",
                name, ep.score, ep.reason
            ));
        }
        if graph.entry_points.len() > limit {
            ctx.push_str(&format!(
                "  ... and {} more entry points\n",
                graph.entry_points.len() - limit
            ));
        }
        ctx.push('\n');
    }

    // Language breakdown (from node kinds)
    let mut kind_counts: std::collections::HashMap<&str, usize> = std::collections::HashMap::new();
    for idx in graph.graph.node_indices() {
        let kind = graph.graph[idx].kind.as_str();
        *kind_counts.entry(kind).or_default() += 1;
    }
    if !kind_counts.is_empty() {
        ctx.push_str("## Symbol Kinds\n\n");
        let mut sorted: Vec<_> = kind_counts.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        for (kind, count) in sorted {
            ctx.push_str(&format!("- {}: {}\n", kind, count));
        }
    }

    ctx
}

fn build_community(graph: &KnowledgeGraph, community_id: CommunityId) -> String {
    let community = match graph.communities.iter().find(|c| c.id == community_id) {
        Some(c) => c,
        None => return format!("Community {} not found.", community_id.0),
    };

    let mut ctx = format!("# Community: {}\n\n", community.label);
    ctx.push_str(&format!(
        "- **ID:** {}\n- **Members:** {}\n- **Cohesion:** {:.2}\n\n",
        community.id.0,
        community.members.len(),
        community.cohesion
    ));

    // Member listing
    ctx.push_str("## Members\n\n");
    let limit = 50.min(community.members.len());
    for sid in &community.members[..limit] {
        if let Some(&idx) = graph.node_index.get(sid) {
            let node = &graph.graph[idx];
            ctx.push_str(&format!("- **{}** ({})\n", node.name, node.kind));
        }
    }
    if community.members.len() > limit {
        ctx.push_str(&format!(
            "\n... and {} more members\n",
            community.members.len() - limit
        ));
    }

    // Inter-community edges
    let member_set: std::collections::HashSet<_> = community
        .members
        .iter()
        .filter_map(|sid| graph.node_index.get(sid).copied())
        .collect();

    let mut external_edges = Vec::new();
    for &idx in &member_set {
        for edge in graph.graph.edges(idx) {
            let target = edge.target();
            if !member_set.contains(&target) {
                let src_name = &graph.graph[idx].name;
                let tgt_name = &graph.graph[target].name;
                external_edges.push(format!("{} → {}", src_name, tgt_name));
            }
        }
    }

    if !external_edges.is_empty() {
        ctx.push_str("\n## External Dependencies\n\n");
        let limit = 20.min(external_edges.len());
        for edge in &external_edges[..limit] {
            ctx.push_str(&format!("- {}\n", edge));
        }
        if external_edges.len() > limit {
            ctx.push_str(&format!(
                "\n... and {} more external edges\n",
                external_edges.len() - limit
            ));
        }
    }

    ctx
}

fn build_symbol(graph: &KnowledgeGraph, symbol_id: SymbolId) -> String {
    let idx = match graph.node_index.get(&symbol_id) {
        Some(&idx) => idx,
        None => return format!("Symbol {} not found.", symbol_id.0),
    };

    let node = &graph.graph[idx];
    let mut ctx = format!("# Symbol: {}\n\n", node.name);
    ctx.push_str(&format!(
        "- **Kind:** {}\n- **Symbol ID:** {}\n- **File ID:** {}\n",
        node.kind, node.symbol_id.0, node.file_id.0
    ));

    // Community
    if let Some(cid) = &node.community_id {
        if let Some(community) = graph.communities.iter().find(|c| c.id == *cid) {
            ctx.push_str(&format!("- **Community:** {}\n", community.label));
        }
    }
    ctx.push('\n');

    // Callers (incoming edges)
    let mut callers = Vec::new();
    for edge in graph.graph.edges_directed(idx, petgraph::Direction::Incoming) {
        let src = edge.source();
        callers.push(graph.graph[src].name.clone());
    }
    if !callers.is_empty() {
        ctx.push_str("## Called By\n\n");
        let limit = 20.min(callers.len());
        for caller in &callers[..limit] {
            ctx.push_str(&format!("- {}\n", caller));
        }
        if callers.len() > limit {
            ctx.push_str(&format!("... and {} more callers\n", callers.len() - limit));
        }
        ctx.push('\n');
    }

    // Callees (outgoing edges)
    let mut callees = Vec::new();
    for edge in graph.graph.edges(idx) {
        let tgt = edge.target();
        callees.push(graph.graph[tgt].name.clone());
    }
    if !callees.is_empty() {
        ctx.push_str("## Calls\n\n");
        let limit = 20.min(callees.len());
        for callee in &callees[..limit] {
            ctx.push_str(&format!("- {}\n", callee));
        }
        if callees.len() > limit {
            ctx.push_str(&format!(
                "... and {} more callees\n",
                callees.len() - limit
            ));
        }
        ctx.push('\n');
    }

    ctx
}

fn build_files(graph: &KnowledgeGraph, paths: &[String]) -> String {
    let mut ctx = String::from("# File Context\n\n");

    // Find nodes that belong to the specified files
    // We use file_id matching via graph nodes
    let mut file_symbols: std::collections::HashMap<&str, Vec<&str>> =
        std::collections::HashMap::new();

    for idx in graph.graph.node_indices() {
        let node = &graph.graph[idx];
        // Match by checking if any path is a substring of the node's file info
        // Since we only have file_id, we list symbols grouped by file_id
        for path in paths {
            // We can't directly match file paths from the graph (only file_ids),
            // so we list all symbols and let the caller filter.
            // For now, include all symbols as context.
            let _ = path;
        }
        file_symbols
            .entry("all")
            .or_default()
            .push(&node.name);
    }

    ctx.push_str(&format!("Requested files: {}\n\n", paths.join(", ")));

    ctx.push_str("## Symbols in Graph\n\n");
    let all_names: Vec<_> = graph
        .graph
        .node_indices()
        .map(|idx| {
            let n = &graph.graph[idx];
            format!("{} ({})", n.name, n.kind)
        })
        .take(100)
        .collect();

    for name in &all_names {
        ctx.push_str(&format!("- {}\n", name));
    }
    if graph.graph.node_count() > 100 {
        ctx.push_str(&format!(
            "\n... and {} more symbols\n",
            graph.graph.node_count() - 100
        ));
    }

    ctx
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeilus_core::ids::{CommunityId, FileId, SymbolId};
    use codeilus_core::types::{Confidence, EdgeKind};
    use codeilus_graph::types::{Community, EntryPoint, GraphEdge, GraphNode};
    use petgraph::graph::DiGraph;
    use std::collections::HashMap;

    fn make_test_graph() -> KnowledgeGraph {
        let mut graph = DiGraph::new();
        let mut node_index = HashMap::new();

        let a = graph.add_node(GraphNode {
            symbol_id: SymbolId(1),
            file_id: FileId(1),
            name: "main".to_string(),
            kind: "Function".to_string(),
            community_id: Some(CommunityId(1)),
        });
        let b = graph.add_node(GraphNode {
            symbol_id: SymbolId(2),
            file_id: FileId(1),
            name: "parse_file".to_string(),
            kind: "Function".to_string(),
            community_id: Some(CommunityId(1)),
        });
        let c = graph.add_node(GraphNode {
            symbol_id: SymbolId(3),
            file_id: FileId(2),
            name: "HttpServer".to_string(),
            kind: "Struct".to_string(),
            community_id: Some(CommunityId(2)),
        });

        node_index.insert(SymbolId(1), a);
        node_index.insert(SymbolId(2), b);
        node_index.insert(SymbolId(3), c);

        graph.add_edge(
            a,
            b,
            GraphEdge {
                kind: EdgeKind::Calls,
                confidence: Confidence::certain(),
            },
        );
        graph.add_edge(
            a,
            c,
            GraphEdge {
                kind: EdgeKind::Calls,
                confidence: Confidence::high(),
            },
        );

        let communities = vec![
            Community {
                id: CommunityId(1),
                label: "Core Parser".to_string(),
                members: vec![SymbolId(1), SymbolId(2)],
                cohesion: 0.9,
            },
            Community {
                id: CommunityId(2),
                label: "HTTP Layer".to_string(),
                members: vec![SymbolId(3)],
                cohesion: 1.0,
            },
        ];

        let entry_points = vec![EntryPoint {
            symbol_id: SymbolId(1),
            score: 0.95,
            reason: "main function".to_string(),
        }];

        KnowledgeGraph {
            graph,
            node_index,
            communities,
            processes: vec![],
            entry_points,
        }
    }

    #[test]
    fn context_overview() {
        let graph = make_test_graph();
        let ctx = build_context(&graph, ContextFocus::Overview);

        assert!(ctx.contains("Repository Overview"));
        assert!(ctx.contains("Symbols:"));
        assert!(ctx.contains("Communities"));
        assert!(ctx.contains("Core Parser"));
        assert!(ctx.contains("HTTP Layer"));
        assert!(ctx.contains("Entry Points"));
        assert!(ctx.contains("main"));
    }

    #[test]
    fn context_overview_truncation() {
        // Build a large graph
        let mut graph = DiGraph::new();
        let mut node_index = HashMap::new();
        let mut members = Vec::new();

        for i in 0..500 {
            let sid = SymbolId(i);
            let idx = graph.add_node(GraphNode {
                symbol_id: sid,
                file_id: FileId(1),
                name: format!("very_long_function_name_for_testing_truncation_{}", i),
                kind: "Function".to_string(),
                community_id: Some(CommunityId(1)),
            });
            node_index.insert(sid, idx);
            members.push(sid);
        }

        let communities = vec![Community {
            id: CommunityId(1),
            label: "Large Module".to_string(),
            members,
            cohesion: 0.5,
        }];

        let kg = KnowledgeGraph {
            graph,
            node_index,
            communities,
            processes: vec![],
            entry_points: vec![],
        };

        let ctx = build_context(&kg, ContextFocus::Overview);
        assert!(
            ctx.len() <= MAX_CONTEXT_CHARS + 100,
            "Context should respect token budget, got {} chars",
            ctx.len()
        );
    }

    #[test]
    fn context_symbol() {
        let graph = make_test_graph();
        let ctx = build_context(&graph, ContextFocus::Symbol(1));

        assert!(ctx.contains("Symbol: main"));
        assert!(ctx.contains("Calls"));
        assert!(ctx.contains("parse_file"));
        assert!(ctx.contains("HttpServer"));
    }

    #[test]
    fn context_community() {
        let graph = make_test_graph();
        let ctx = build_context(&graph, ContextFocus::Community(1));

        assert!(ctx.contains("Community: Core Parser"));
        assert!(ctx.contains("main"));
        assert!(ctx.contains("parse_file"));
        assert!(ctx.contains("Members"));
    }
}
