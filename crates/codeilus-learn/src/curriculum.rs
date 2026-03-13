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
    let chapter0 = make_chapter(
        ChapterId(chapter_counter),
        0,
        "The Big Picture",
        "An overview of the entire codebase — architecture, key components, and how everything fits together.",
        None,
        Difficulty::Beginner,
        vec![],
    );
    chapter_counter += 1;
    chapters.push(chapter0);

    // Build community dependency graph and topological sort
    let sorted_communities = topological_sort_communities(graph);

    // Map community_id → chapter_id for prerequisite tracking
    let mut community_to_chapter: HashMap<i64, ChapterId> = HashMap::new();

    for (order, community_id) in sorted_communities.iter().enumerate() {
        let community = graph
            .communities
            .iter()
            .find(|c| c.id.0 == *community_id);
        let community = match community {
            Some(c) => c,
            None => continue,
        };

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

        let chapter = make_chapter(
            chapter_id,
            order + 1,
            &community.label,
            &format!("Deep dive into the {} module.", community.label),
            Some(codeilus_core::ids::CommunityId(*community_id)),
            difficulty,
            prerequisite_ids,
        );

        community_to_chapter.insert(*community_id, chapter_id);
        chapters.push(chapter);
    }

    // Final chapter: Putting It All Together (always last)
    let final_order = chapters.len();
    let all_prereqs: Vec<ChapterId> = chapters.iter().map(|c| c.id).collect();
    let final_chapter = make_chapter(
        ChapterId(chapter_counter),
        final_order,
        "Putting It All Together",
        "Cross-cutting execution flows, how modules interact, and the complete picture.",
        None,
        Difficulty::Intermediate,
        all_prereqs,
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

fn make_chapter(
    id: ChapterId,
    order: usize,
    title: &str,
    description: &str,
    community_id: Option<codeilus_core::ids::CommunityId>,
    difficulty: Difficulty,
    prerequisite_ids: Vec<ChapterId>,
) -> Chapter {
    let sections: Vec<Section> = SectionKind::all()
        .iter()
        .map(|kind| Section {
            id: kind.as_str().to_string(),
            title: kind.title().to_string(),
            kind: *kind,
        })
        .collect();

    Chapter {
        id,
        order,
        title: title.to_string(),
        description: description.to_string(),
        community_id,
        sections,
        difficulty,
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
            for n in 0..3 {
                let sid = SymbolId((c * 3 + n) as i64);
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
        let tgt = *kg.node_index.get(&SymbolId(3)).unwrap(); // community 1
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
        // 3 members → Beginner
        assert_eq!(community_chapters[0].difficulty, Difficulty::Beginner);
    }

    #[test]
    fn curriculum_prerequisites() {
        // Community 0 depends on Community 1
        let mut kg = make_test_graph(2);
        let src = *kg.node_index.get(&SymbolId(0)).unwrap();
        let tgt = *kg.node_index.get(&SymbolId(3)).unwrap();
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
