//! Curriculum generation: topological sort of communities → ordered chapters.

use codeilus_core::ids::ChapterId;
use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use std::collections::{HashMap, HashSet, VecDeque};
use tracing::debug;

use crate::types::{Chapter, Curriculum, Difficulty, Section, SectionKind};

/// Generate a curriculum from the knowledge graph.
pub fn generate(graph: &KnowledgeGraph) -> CodeilusResult<Curriculum> {
    let mut chapters = Vec::new();
    let mut chapter_counter: i64 = 1;

    // Chapter 0: The Big Picture (always first, no community)
    let chapter0 = make_big_picture_chapter(ChapterId(chapter_counter), graph);
    chapter_counter += 1;
    chapters.push(chapter0);

    // Build community dependency graph and topological sort
    let sorted_communities = topological_sort_communities(graph);

    // Map community_id → chapter_id for prerequisite tracking
    let mut community_to_chapter: HashMap<i64, ChapterId> = HashMap::new();

    // Filter: only create chapters for communities with enough members to be meaningful
    let min_community_size = 5;

    for (order, community_id) in sorted_communities.iter().enumerate() {
        let community = graph
            .communities
            .iter()
            .find(|c| c.id.0 == *community_id);
        let community = match community {
            Some(c) => c,
            None => continue,
        };

        // Skip tiny communities — they don't warrant their own chapter
        if community.members.len() < min_community_size {
            continue;
        }

        let chapter_id = ChapterId(chapter_counter);
        chapter_counter += 1;

        // Determine difficulty based on community size (proxy for complexity)
        let difficulty = estimate_difficulty(community.members.len());

        // Determine prerequisites from community dependencies
        let deps = get_community_dependencies(graph, *community_id);
        let prerequisite_ids: Vec<ChapterId> = deps
            .iter()
            .filter_map(|dep_id| community_to_chapter.get(dep_id).copied())
            .collect();

        let chapter = make_community_chapter(
            chapter_id,
            order + 1,
            community,
            difficulty,
            prerequisite_ids.clone(),
            graph,
            &community_to_chapter,
        );

        community_to_chapter.insert(*community_id, chapter_id);
        chapters.push(chapter);
    }

    // Final chapter: Putting It All Together (always last)
    let final_order = chapters.len();
    let all_prereqs: Vec<ChapterId> = chapters.iter().map(|c| c.id).collect();
    let final_chapter = make_final_chapter(
        ChapterId(chapter_counter),
        final_order,
        all_prereqs,
        graph,
    );
    chapters.push(final_chapter);

    let total_sections = chapters.iter().map(|c| c.sections.len()).sum();

    debug!(
        chapters = chapters.len(),
        total_sections, "generated curriculum"
    );

    Ok(Curriculum {
        chapters,
        total_sections,
    })
}

fn make_section(kind: SectionKind, content: String) -> Section {
    Section {
        id: kind.as_str().to_string(),
        title: kind.title().to_string(),
        kind,
        content,
    }
}

/// Build Chapter 0: "The Big Picture" with real content from the graph.
fn make_big_picture_chapter(id: ChapterId, graph: &KnowledgeGraph) -> Chapter {
    let total_nodes = graph.graph.node_count();
    let total_edges = graph.graph.edge_count();
    let total_communities = graph.communities.len();

    // Overview: architecture summary
    let overview = {
        let mut lines = vec![format!(
            "This codebase contains **{total_nodes}** symbols organised into \
             **{total_communities}** modules, connected by **{total_edges}** relationships."
        )];
        if !graph.entry_points.is_empty() {
            lines.push("\n### Entry Points".to_string());
            for ep in graph.entry_points.iter().take(10) {
                if let Some(idx) = graph.node_index.get(&ep.symbol_id) {
                    let node = &graph.graph[*idx];
                    lines.push(format!(
                        "- **{}** (score {:.1}) — {}",
                        node.name, ep.score, ep.reason
                    ));
                }
            }
        }
        lines.join("\n")
    };

    // Key Concepts: list communities with member counts
    let key_concepts = {
        let mut lines = vec!["### Modules at a Glance".to_string()];
        for comm in &graph.communities {
            lines.push(format!(
                "- **{}** — {} symbols (cohesion {:.0}%)",
                comm.label,
                comm.members.len(),
                comm.cohesion * 100.0
            ));
        }
        lines.join("\n")
    };

    // Diagram placeholder
    let diagram = "See the interactive dependency diagram for a visual map of all modules.".to_string();

    // Code Walkthrough: top-level reading order by in-degree (most-imported first)
    let code_walkthrough = {
        let mut in_degrees: Vec<(String, usize)> = graph
            .graph
            .node_indices()
            .map(|idx| {
                let name = graph.graph[idx].name.clone();
                let deg = graph
                    .graph
                    .neighbors_directed(idx, petgraph::Direction::Incoming)
                    .count();
                (name, deg)
            })
            .collect();
        in_degrees.sort_by(|a, b| b.1.cmp(&a.1));
        let mut lines = vec!["### Suggested Reading Order (most-imported first)".to_string()];
        for (i, (name, deg)) in in_degrees.iter().take(15).enumerate() {
            lines.push(format!("{}. **{}** — imported by {} others", i + 1, name, deg));
        }
        lines.join("\n")
    };

    // Connections: inter-module edges summary
    let connections = {
        let mut lines = vec!["### How Modules Connect".to_string()];
        for comm in &graph.communities {
            let dep_ids = get_community_dependencies(graph, comm.id.0);
            if dep_ids.is_empty() {
                lines.push(format!("- **{}** — standalone (no outgoing module deps)", comm.label));
            } else {
                let dep_labels: Vec<String> = dep_ids
                    .iter()
                    .filter_map(|did| graph.communities.iter().find(|c| c.id.0 == *did))
                    .map(|c| c.label.clone())
                    .collect();
                lines.push(format!("- **{}** → {}", comm.label, dep_labels.join(", ")));
            }
        }
        lines.join("\n")
    };

    let quiz_content = "Complete the quiz to test your understanding of the big picture.".to_string();

    Chapter {
        id,
        order: 0,
        title: "The Big Picture".to_string(),
        description: "An overview of the entire codebase — architecture, key components, and how everything fits together.".to_string(),
        community_id: None,
        sections: vec![
            make_section(SectionKind::Overview, overview),
            make_section(SectionKind::KeyConcepts, key_concepts),
            make_section(SectionKind::Diagram, diagram),
            make_section(SectionKind::CodeWalkthrough, code_walkthrough),
            make_section(SectionKind::Connections, connections),
            make_section(SectionKind::Quiz, quiz_content),
        ],
        difficulty: Difficulty::Beginner,
        prerequisite_ids: vec![],
    }
}

/// Build a chapter for a specific community with populated content.
fn make_community_chapter(
    id: ChapterId,
    order: usize,
    community: &codeilus_graph::Community,
    difficulty: Difficulty,
    prerequisite_ids: Vec<ChapterId>,
    graph: &KnowledgeGraph,
    community_to_chapter: &HashMap<i64, ChapterId>,
) -> Chapter {
    // Overview: list key symbols and their roles
    let overview = {
        let mut lines = vec![format!(
            "The **{}** module contains **{}** symbols with a cohesion of {:.0}%.",
            community.label,
            community.members.len(),
            community.cohesion * 100.0
        )];
        lines.push("\n### Key Symbols".to_string());
        for &sid in community.members.iter().take(20) {
            if let Some(idx) = graph.node_index.get(&sid) {
                let node = &graph.graph[*idx];
                lines.push(format!("- **{}** ({})", node.name, node.kind));
            }
        }
        lines.join("\n")
    };

    // Key Concepts: group members by kind
    let key_concepts = {
        let mut by_kind: HashMap<&str, Vec<&str>> = HashMap::new();
        for &sid in &community.members {
            if let Some(idx) = graph.node_index.get(&sid) {
                let node = &graph.graph[*idx];
                by_kind.entry(node.kind.as_str()).or_default().push(node.name.as_str());
            }
        }
        let mut lines = vec!["### Symbols by Kind".to_string()];
        let mut kinds: Vec<_> = by_kind.into_iter().collect();
        kinds.sort_by_key(|(k, _)| *k);
        for (kind, names) in kinds {
            lines.push(format!("- **{}**: {}", kind, names.join(", ")));
        }
        lines.join("\n")
    };

    let diagram = format!(
        "See the interactive diagram for the **{}** module and its internal structure.",
        community.label
    );

    // Code Walkthrough: ordered by in-degree within community (most-imported first)
    let code_walkthrough = {
        let member_set: HashSet<petgraph::graph::NodeIndex> = community
            .members
            .iter()
            .filter_map(|sid| graph.node_index.get(sid).copied())
            .collect();
        let mut in_degrees: Vec<(String, String, usize)> = community
            .members
            .iter()
            .filter_map(|sid| graph.node_index.get(sid).map(|idx| (sid, *idx)))
            .map(|(_sid, idx)| {
                let node = &graph.graph[idx];
                let deg = graph
                    .graph
                    .neighbors_directed(idx, petgraph::Direction::Incoming)
                    .filter(|n| member_set.contains(n))
                    .count();
                (node.name.clone(), node.kind.clone(), deg)
            })
            .collect();
        in_degrees.sort_by(|a, b| b.2.cmp(&a.2));
        let mut lines = vec!["### Reading Order (most-referenced first)".to_string()];
        for (i, (name, kind, deg)) in in_degrees.iter().enumerate() {
            lines.push(format!(
                "{}. **{}** ({}) — referenced by {} members",
                i + 1, name, kind, deg
            ));
        }
        lines.join("\n")
    };

    // Connections: which other chapters/modules this one depends on
    let connections = {
        let dep_ids = get_community_dependencies(graph, community.id.0);
        let mut lines = vec!["### Dependencies".to_string()];
        if dep_ids.is_empty() {
            lines.push("This module has no outgoing dependencies on other modules.".to_string());
        } else {
            for did in &dep_ids {
                let label = graph
                    .communities
                    .iter()
                    .find(|c| c.id.0 == *did)
                    .map(|c| c.label.as_str())
                    .unwrap_or("unknown");
                let chapter_ref = community_to_chapter
                    .get(did)
                    .map(|cid| format!(" (Chapter {})", cid.0))
                    .unwrap_or_default();
                lines.push(format!("- Depends on **{}**{}", label, chapter_ref));
            }
        }
        // Also show who depends on us (reverse)
        let dependents: Vec<&str> = graph
            .communities
            .iter()
            .filter(|c| c.id.0 != community.id.0)
            .filter(|c| {
                get_community_dependencies(graph, c.id.0).contains(&community.id.0)
            })
            .map(|c| c.label.as_str())
            .collect();
        if !dependents.is_empty() {
            lines.push("\n### Dependents".to_string());
            for d in &dependents {
                lines.push(format!("- **{}** depends on this module", d));
            }
        }
        lines.join("\n")
    };

    let quiz_content = format!(
        "Complete the quiz to test your understanding of the **{}** module.",
        community.label
    );

    Chapter {
        id,
        order,
        title: community.label.clone(),
        description: format!("Deep dive into the {} module.", community.label),
        community_id: Some(community.id),
        sections: vec![
            make_section(SectionKind::Overview, overview),
            make_section(SectionKind::KeyConcepts, key_concepts),
            make_section(SectionKind::Diagram, diagram),
            make_section(SectionKind::CodeWalkthrough, code_walkthrough),
            make_section(SectionKind::Connections, connections),
            make_section(SectionKind::Quiz, quiz_content),
        ],
        difficulty,
        prerequisite_ids,
    }
}

/// Build the final chapter: "Putting It All Together" with cross-cutting data flow.
fn make_final_chapter(
    id: ChapterId,
    order: usize,
    prerequisite_ids: Vec<ChapterId>,
    graph: &KnowledgeGraph,
) -> Chapter {
    // Overview: data flow summary from processes
    let overview = {
        let mut lines = vec![
            "Now that you've explored each module, let's see how they work together.".to_string(),
        ];
        if !graph.processes.is_empty() {
            lines.push("\n### Execution Flows".to_string());
            for proc in &graph.processes {
                lines.push(format!("- **{}**", proc.name));
                for step in &proc.steps {
                    if let Some(idx) = graph.node_index.get(&step.symbol_id) {
                        let node = &graph.graph[*idx];
                        lines.push(format!(
                            "  {}. `{}` — {}",
                            step.order + 1,
                            node.name,
                            step.description
                        ));
                    }
                }
            }
        }
        lines.join("\n")
    };

    // Key Concepts: cross-module edge summary
    let key_concepts = {
        let mut cross_edges: HashMap<(String, String), usize> = HashMap::new();
        for edge_idx in graph.graph.edge_indices() {
            if let Some((src, tgt)) = graph.graph.edge_endpoints(edge_idx) {
                let src_comm = graph.graph[src].community_id;
                let tgt_comm = graph.graph[tgt].community_id;
                if let (Some(sc), Some(tc)) = (src_comm, tgt_comm) {
                    if sc != tc {
                        let sl = graph.communities.iter().find(|c| c.id == sc).map(|c| c.label.clone()).unwrap_or_default();
                        let tl = graph.communities.iter().find(|c| c.id == tc).map(|c| c.label.clone()).unwrap_or_default();
                        *cross_edges.entry((sl, tl)).or_insert(0) += 1;
                    }
                }
            }
        }
        let mut lines = vec!["### Cross-Module Relationships".to_string()];
        let mut sorted: Vec<_> = cross_edges.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        for ((from, to), count) in sorted.iter().take(20) {
            lines.push(format!("- **{}** → **{}** ({} edges)", from, to, count));
        }
        if sorted.is_empty() {
            lines.push("No cross-module edges detected.".to_string());
        }
        lines.join("\n")
    };

    let diagram = "See the full cross-module dependency diagram.".to_string();

    // Walkthrough: entry points as starting reading order
    let code_walkthrough = {
        let mut lines = vec!["### Start-to-Finish Reading Path".to_string()];
        if graph.entry_points.is_empty() {
            lines.push("Follow the chapter order for the recommended reading path.".to_string());
        } else {
            for (i, ep) in graph.entry_points.iter().take(10).enumerate() {
                if let Some(idx) = graph.node_index.get(&ep.symbol_id) {
                    let node = &graph.graph[*idx];
                    let comm_label = node
                        .community_id
                        .and_then(|cid| graph.communities.iter().find(|c| c.id == cid))
                        .map(|c| c.label.as_str())
                        .unwrap_or("—");
                    lines.push(format!(
                        "{}. **{}** in *{}* — {}",
                        i + 1,
                        node.name,
                        comm_label,
                        ep.reason
                    ));
                }
            }
        }
        lines.join("\n")
    };

    let connections =
        "All modules are connected. Review the Connections sections of earlier chapters for per-module details."
            .to_string();

    let quiz_content = "Final quiz: test your understanding of cross-cutting flows.".to_string();

    Chapter {
        id,
        order,
        title: "Putting It All Together".to_string(),
        description: "Cross-cutting execution flows, how modules interact, and the complete picture.".to_string(),
        community_id: None,
        sections: vec![
            make_section(SectionKind::Overview, overview),
            make_section(SectionKind::KeyConcepts, key_concepts),
            make_section(SectionKind::Diagram, diagram),
            make_section(SectionKind::CodeWalkthrough, code_walkthrough),
            make_section(SectionKind::Connections, connections),
            make_section(SectionKind::Quiz, quiz_content),
        ],
        difficulty: Difficulty::Intermediate,
        prerequisite_ids,
    }
}

fn estimate_difficulty(member_count: usize) -> Difficulty {
    if member_count < 5 {
        Difficulty::Beginner
    } else if member_count < 15 {
        Difficulty::Intermediate
    } else {
        Difficulty::Advanced
    }
}

/// Build a dependency graph between communities and return topological order.
/// Community A depends on B if any node in A has an edge to a node in B.
fn topological_sort_communities(graph: &KnowledgeGraph) -> Vec<i64> {
    // Build community_id → set of member node indices
    let mut community_members: HashMap<i64, HashSet<petgraph::graph::NodeIndex>> = HashMap::new();
    let mut node_to_community: HashMap<petgraph::graph::NodeIndex, i64> = HashMap::new();

    for node_idx in graph.graph.node_indices() {
        let node = &graph.graph[node_idx];
        if let Some(cid) = &node.community_id {
            community_members
                .entry(cid.0)
                .or_default()
                .insert(node_idx);
            node_to_community.insert(node_idx, cid.0);
        }
    }

    let community_ids: Vec<i64> = community_members.keys().copied().collect();
    if community_ids.is_empty() {
        return vec![];
    }

    // Build inter-community dependency edges
    let mut deps: HashMap<i64, HashSet<i64>> = HashMap::new();
    let mut in_degree: HashMap<i64, usize> = HashMap::new();
    for &cid in &community_ids {
        deps.entry(cid).or_default();
        in_degree.entry(cid).or_insert(0);
    }

    for edge_idx in graph.graph.edge_indices() {
        if let Some((src, tgt)) = graph.graph.edge_endpoints(edge_idx) {
            let src_comm = node_to_community.get(&src);
            let tgt_comm = node_to_community.get(&tgt);
            if let (Some(&sc), Some(&tc)) = (src_comm, tgt_comm) {
                if sc != tc && !deps.entry(sc).or_default().contains(&tc) {
                    // sc depends on tc → tc must come before sc in toposort
                    deps.entry(sc).or_default().insert(tc);
                    *in_degree.entry(sc).or_insert(0) += 1;
                }
            }
        }
    }

    // Entry point communities get priority (sort by entry point score)
    let entry_communities: HashSet<i64> = graph
        .entry_points
        .iter()
        .filter_map(|ep| {
            graph
                .node_index
                .get(&ep.symbol_id)
                .and_then(|idx| node_to_community.get(idx).copied())
        })
        .collect();

    // Kahn's algorithm for topological sort
    let mut queue: VecDeque<i64> = VecDeque::new();
    for &cid in &community_ids {
        if in_degree[&cid] == 0 {
            queue.push_back(cid);
        }
    }

    // Sort queue: entry point communities first
    let mut sorted_queue: Vec<i64> = queue.drain(..).collect();
    sorted_queue.sort_by(|a, b| {
        let a_entry = entry_communities.contains(a);
        let b_entry = entry_communities.contains(b);
        b_entry.cmp(&a_entry).then(a.cmp(b))
    });
    queue.extend(sorted_queue);

    let mut result = Vec::new();
    while let Some(cid) = queue.pop_front() {
        result.push(cid);
        let dependents: Vec<i64> = deps
            .iter()
            .filter_map(|(&from, to_set)| {
                if to_set.contains(&cid) {
                    Some(from)
                } else {
                    None
                }
            })
            .collect();
        for dep in dependents {
            if let Some(deg) = in_degree.get_mut(&dep) {
                *deg = deg.saturating_sub(1);
                if *deg == 0 {
                    queue.push_back(dep);
                }
            }
        }
    }

    // If there are cycles, add remaining communities
    for &cid in &community_ids {
        if !result.contains(&cid) {
            result.push(cid);
        }
    }

    result
}

/// Get community IDs that a given community depends on (has edges to).
fn get_community_dependencies(graph: &KnowledgeGraph, community_id: i64) -> Vec<i64> {
    let mut deps = HashSet::new();
    let mut node_to_community: HashMap<petgraph::graph::NodeIndex, i64> = HashMap::new();

    for node_idx in graph.graph.node_indices() {
        let node = &graph.graph[node_idx];
        if let Some(cid) = &node.community_id {
            node_to_community.insert(node_idx, cid.0);
        }
    }

    for edge_idx in graph.graph.edge_indices() {
        if let Some((src, tgt)) = graph.graph.edge_endpoints(edge_idx) {
            let src_comm = node_to_community.get(&src).copied();
            let tgt_comm = node_to_community.get(&tgt).copied();
            if src_comm == Some(community_id) {
                if let Some(tc) = tgt_comm {
                    if tc != community_id {
                        deps.insert(tc);
                    }
                }
            }
        }
    }

    deps.into_iter().collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeilus_core::ids::{CommunityId, FileId, SymbolId};
    use codeilus_core::types::{Confidence, EdgeKind};
    use codeilus_graph::{Community, GraphEdge, GraphNode, KnowledgeGraph};
    use petgraph::graph::DiGraph;

    fn make_test_graph(num_communities: usize) -> KnowledgeGraph {
        let mut graph = DiGraph::new();
        let mut node_index = HashMap::new();
        let mut communities = Vec::new();

        for c in 0..num_communities {
            let cid = CommunityId(c as i64);
            let mut members = Vec::new();
            for n in 0..6 {
                let sid = SymbolId((c * 6 + n) as i64);
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
    fn curriculum_has_chapter_zero() {
        let kg = make_test_graph(2);
        let curriculum = generate(&kg).unwrap();
        assert_eq!(curriculum.chapters[0].title, "The Big Picture");
        assert_eq!(curriculum.chapters[0].order, 0);
    }

    #[test]
    fn curriculum_has_final_chapter() {
        let kg = make_test_graph(2);
        let curriculum = generate(&kg).unwrap();
        let last = curriculum.chapters.last().unwrap();
        assert_eq!(last.title, "Putting It All Together");
    }

    #[test]
    fn curriculum_topological_order() {
        // Community 0 depends on Community 1 (has edge from 0→1)
        let mut kg = make_test_graph(2);
        let src = *kg.node_index.get(&SymbolId(0)).unwrap(); // community 0
        let tgt = *kg.node_index.get(&SymbolId(6)).unwrap(); // community 1
        kg.graph.add_edge(
            src,
            tgt,
            GraphEdge {
                kind: EdgeKind::Calls,
                confidence: Confidence::high(),
            },
        );
        let curriculum = generate(&kg).unwrap();

        // Find chapter positions for communities
        let comm0_pos = curriculum
            .chapters
            .iter()
            .position(|c| c.community_id == Some(CommunityId(0)))
            .unwrap();
        let comm1_pos = curriculum
            .chapters
            .iter()
            .position(|c| c.community_id == Some(CommunityId(1)))
            .unwrap();

        // Community 1 (dependency) should come before Community 0 (dependent)
        assert!(
            comm1_pos < comm0_pos,
            "Dependency (comm 1 at {}) should come before dependent (comm 0 at {})",
            comm1_pos,
            comm0_pos
        );
    }

    #[test]
    fn curriculum_sections_complete() {
        let kg = make_test_graph(1);
        let curriculum = generate(&kg).unwrap();
        for chapter in &curriculum.chapters {
            assert_eq!(
                chapter.sections.len(),
                6,
                "Chapter '{}' should have 6 sections, got {}",
                chapter.title,
                chapter.sections.len()
            );
            // Check all section kinds are present
            let kinds: Vec<SectionKind> = chapter.sections.iter().map(|s| s.kind).collect();
            assert!(kinds.contains(&SectionKind::Overview));
            assert!(kinds.contains(&SectionKind::KeyConcepts));
            assert!(kinds.contains(&SectionKind::Diagram));
            assert!(kinds.contains(&SectionKind::CodeWalkthrough));
            assert!(kinds.contains(&SectionKind::Connections));
            assert!(kinds.contains(&SectionKind::Quiz));
        }
    }

    #[test]
    fn curriculum_difficulty_from_complexity() {
        // Community with 2 members → Beginner
        let kg = make_test_graph(1);
        let curriculum = generate(&kg).unwrap();
        // Community chapters (excluding chapter 0 and final)
        let community_chapters: Vec<&Chapter> = curriculum
            .chapters
            .iter()
            .filter(|c| c.community_id.is_some())
            .collect();
        assert!(!community_chapters.is_empty());
        // 6 members → Intermediate
        assert_eq!(community_chapters[0].difficulty, Difficulty::Intermediate);
    }

    #[test]
    fn curriculum_prerequisites() {
        // Community 0 depends on Community 1
        let mut kg = make_test_graph(2);
        let src = *kg.node_index.get(&SymbolId(0)).unwrap();
        let tgt = *kg.node_index.get(&SymbolId(6)).unwrap();
        kg.graph.add_edge(
            src,
            tgt,
            GraphEdge {
                kind: EdgeKind::Calls,
                confidence: Confidence::high(),
            },
        );
        let curriculum = generate(&kg).unwrap();

        let comm0_chapter = curriculum
            .chapters
            .iter()
            .find(|c| c.community_id == Some(CommunityId(0)))
            .unwrap();
        let comm1_chapter = curriculum
            .chapters
            .iter()
            .find(|c| c.community_id == Some(CommunityId(1)))
            .unwrap();

        assert!(
            comm0_chapter.prerequisite_ids.contains(&comm1_chapter.id),
            "Community 0 chapter should list Community 1 chapter as prerequisite"
        );
    }
}
