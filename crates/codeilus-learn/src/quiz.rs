//! Quiz generation from knowledge graph data.

use codeilus_core::ids::ChapterId;
use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use tracing::debug;

use crate::types::{Chapter, Quiz, QuizQuestion, QuizQuestionKind};

const TARGET_QUESTIONS: usize = 5;

/// Generate a quiz for a chapter from graph data.
pub fn generate_quiz(chapter: &Chapter, graph: &KnowledgeGraph) -> CodeilusResult<Quiz> {
    let mut questions = Vec::new();

    // Get community members for this chapter
    let member_symbols: Vec<_> = if let Some(cid) = &chapter.community_id {
        graph
            .communities
            .iter()
            .find(|c| c.id == *cid)
            .map(|c| c.members.clone())
            .unwrap_or_default()
    } else {
        // For Chapter 0 or final chapter, use all symbols
        graph.node_index.keys().copied().collect()
    };

    if member_symbols.is_empty() {
        return Ok(Quiz {
            chapter_id: chapter.id,
            questions: vec![],
        });
    }

    // Generate MultipleChoice questions about dependencies
    questions.extend(generate_dependency_questions(graph, &member_symbols, chapter.id));

    // Generate TrueFalse questions about call relationships
    questions.extend(generate_call_questions(graph, &member_symbols, chapter.id));

    // Generate ImpactAnalysis questions
    questions.extend(generate_impact_questions(graph, &member_symbols, chapter.id));

    // Trim to target count or pad with generic questions
    questions.truncate(TARGET_QUESTIONS);
    while questions.len() < TARGET_QUESTIONS {
        questions.push(make_generic_question(
            chapter.id,
            questions.len(),
            &chapter.title,
        ));
    }

    debug!(
        chapter = chapter.title,
        questions = questions.len(),
        "generated quiz"
    );

    Ok(Quiz {
        chapter_id: chapter.id,
        questions,
    })
}

fn generate_dependency_questions(
    graph: &KnowledgeGraph,
    member_symbols: &[codeilus_core::ids::SymbolId],
    chapter_id: ChapterId,
) -> Vec<QuizQuestion> {
    let mut questions = Vec::new();

    for (i, &sid) in member_symbols.iter().take(2).enumerate() {
        let node_idx = match graph.node_index.get(&sid) {
            Some(idx) => *idx,
            None => continue,
        };
        let node = &graph.graph[node_idx];

        // Find what this symbol calls/depends on
        let callees: Vec<String> = graph
            .graph
            .neighbors(node_idx)
            .map(|n| graph.graph[n].name.clone())
            .take(4)
            .collect();

        if callees.is_empty() {
            continue;
        }

        let correct = callees[0].clone();
        let mut options = callees.clone();
        // Add a wrong option
        options.push(format!("{}Helper", node.name));
        options.truncate(4);
        let correct_index = options.iter().position(|o| *o == correct).unwrap_or(0);

        questions.push(QuizQuestion {
            id: format!("q_dep_{}_{}", chapter_id.0, i),
            question: format!("Which symbol does '{}' depend on?", node.name),
            kind: QuizQuestionKind::MultipleChoice,
            options,
            correct_index,
            explanation: format!(
                "'{}' has a direct dependency edge to '{}' in the knowledge graph.",
                node.name, correct
            ),
        });
    }

    questions
}

fn generate_call_questions(
    graph: &KnowledgeGraph,
    member_symbols: &[codeilus_core::ids::SymbolId],
    chapter_id: ChapterId,
) -> Vec<QuizQuestion> {
    let mut questions = Vec::new();

    for (i, &sid) in member_symbols.iter().take(2).enumerate() {
        let node_idx = match graph.node_index.get(&sid) {
            Some(idx) => *idx,
            None => continue,
        };
        let node = &graph.graph[node_idx];
        let has_callee = graph.graph.neighbors(node_idx).next().is_some();

        if has_callee {
            let callee = &graph.graph[graph.graph.neighbors(node_idx).next().unwrap()];
            questions.push(QuizQuestion {
                id: format!("q_call_{}_{}", chapter_id.0, i),
                question: format!("True or False: '{}' calls '{}'.", node.name, callee.name),
                kind: QuizQuestionKind::TrueFalse,
                options: vec!["True".to_string(), "False".to_string()],
                correct_index: 0,
                explanation: format!(
                    "'{}' calls '{}' as shown by the CALLS edge in the knowledge graph.",
                    node.name, callee.name
                ),
            });
        } else {
            // No callees — make a false statement
            let fake_callee = format!("{}Processor", node.name);
            questions.push(QuizQuestion {
                id: format!("q_call_{}_{}", chapter_id.0, i),
                question: format!(
                    "True or False: '{}' calls '{}'.",
                    node.name, fake_callee
                ),
                kind: QuizQuestionKind::TrueFalse,
                options: vec!["True".to_string(), "False".to_string()],
                correct_index: 1,
                explanation: format!(
                    "'{}' does not call '{}' — there is no such edge in the knowledge graph.",
                    node.name, fake_callee
                ),
            });
        }
    }

    questions
}

fn generate_impact_questions(
    graph: &KnowledgeGraph,
    member_symbols: &[codeilus_core::ids::SymbolId],
    chapter_id: ChapterId,
) -> Vec<QuizQuestion> {
    let mut questions = Vec::new();

    for &sid in member_symbols.iter().take(1) {
        let node_idx = match graph.node_index.get(&sid) {
            Some(idx) => *idx,
            None => continue,
        };
        let node = &graph.graph[node_idx];

        // Find callers (reverse edges)
        let callers: Vec<String> = graph
            .graph
            .neighbors_directed(node_idx, petgraph::Direction::Incoming)
            .map(|n| graph.graph[n].name.clone())
            .take(4)
            .collect();

        if callers.is_empty() {
            continue;
        }

        let mut options = callers.clone();
        options.push("No other functions".to_string());
        let correct_answer = callers.join(", ");
        let correct_index = 0; // First option includes the correct callers

        questions.push(QuizQuestion {
            id: format!("q_impact_{}_{}", chapter_id.0, 0),
            question: format!(
                "If you change '{}', which functions would be affected?",
                node.name
            ),
            kind: QuizQuestionKind::ImpactAnalysis,
            options,
            correct_index,
            explanation: format!(
                "Changing '{}' would affect: {}. These functions have direct dependency edges to '{}'.",
                node.name, correct_answer, node.name
            ),
        });
    }

    questions
}

fn make_generic_question(
    chapter_id: ChapterId,
    index: usize,
    chapter_title: &str,
) -> QuizQuestion {
    QuizQuestion {
        id: format!("q_gen_{}_{}", chapter_id.0, index),
        question: format!(
            "What is the primary purpose of the '{}' module?",
            chapter_title
        ),
        kind: QuizQuestionKind::MultipleChoice,
        options: vec![
            format!("Core functionality of {}", chapter_title),
            "Database management".to_string(),
            "User interface rendering".to_string(),
            "External API integration".to_string(),
        ],
        correct_index: 0,
        explanation: format!(
            "The '{}' module provides its core functionality as described in the chapter overview.",
            chapter_title
        ),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeilus_core::ids::{CommunityId, FileId, SymbolId};
    use codeilus_core::types::{Confidence, EdgeKind};
    use codeilus_graph::{Community, GraphEdge, GraphNode, KnowledgeGraph};
    use petgraph::graph::DiGraph;
    use std::collections::HashMap;

    fn make_test_graph_with_edges() -> KnowledgeGraph {
        let mut graph = DiGraph::new();
        let mut node_index = HashMap::new();

        let cid = CommunityId(0);
        let mut members = Vec::new();

        for i in 0..5 {
            let sid = SymbolId(i);
            let idx = graph.add_node(GraphNode {
                symbol_id: sid,
                file_id: FileId(0),
                name: format!("func_{}", i),
                kind: "fn".to_string(),
                community_id: Some(cid),
            });
            node_index.insert(sid, idx);
            members.push(sid);
        }

        // Add call edges: 0→1, 0→2, 1→3, 2→4
        let n0 = *node_index.get(&SymbolId(0)).unwrap();
        let n1 = *node_index.get(&SymbolId(1)).unwrap();
        let n2 = *node_index.get(&SymbolId(2)).unwrap();
        let n3 = *node_index.get(&SymbolId(3)).unwrap();
        let n4 = *node_index.get(&SymbolId(4)).unwrap();

        graph.add_edge(n0, n1, GraphEdge { kind: EdgeKind::Calls, confidence: Confidence::high() });
        graph.add_edge(n0, n2, GraphEdge { kind: EdgeKind::Calls, confidence: Confidence::high() });
        graph.add_edge(n1, n3, GraphEdge { kind: EdgeKind::Calls, confidence: Confidence::high() });
        graph.add_edge(n2, n4, GraphEdge { kind: EdgeKind::Calls, confidence: Confidence::high() });

        KnowledgeGraph {
            graph,
            node_index,
            communities: vec![Community {
                id: cid,
                label: "Core".to_string(),
                members,
                cohesion: 0.9,
            }],
            processes: vec![],
            entry_points: vec![],
        }
    }

    fn make_test_chapter() -> Chapter {
        use crate::types::{Difficulty, Section, SectionKind};
        Chapter {
            id: ChapterId(1),
            order: 1,
            title: "Core".to_string(),
            description: "Core module".to_string(),
            community_id: Some(CommunityId(0)),
            sections: SectionKind::all()
                .iter()
                .map(|k| Section {
                    id: k.as_str().to_string(),
                    title: k.title().to_string(),
                    kind: *k,
                })
                .collect(),
            difficulty: Difficulty::Beginner,
            prerequisite_ids: vec![],
        }
    }

    #[test]
    fn quiz_five_questions() {
        let graph = make_test_graph_with_edges();
        let chapter = make_test_chapter();
        let quiz = generate_quiz(&chapter, &graph).unwrap();
        assert_eq!(quiz.questions.len(), 5, "Quiz should have exactly 5 questions");
    }

    #[test]
    fn quiz_correct_index_valid() {
        let graph = make_test_graph_with_edges();
        let chapter = make_test_chapter();
        let quiz = generate_quiz(&chapter, &graph).unwrap();
        for q in &quiz.questions {
            assert!(
                q.correct_index < q.options.len(),
                "correct_index {} should be < options length {} for question: {}",
                q.correct_index,
                q.options.len(),
                q.question
            );
        }
    }

    #[test]
    fn quiz_has_explanation() {
        let graph = make_test_graph_with_edges();
        let chapter = make_test_chapter();
        let quiz = generate_quiz(&chapter, &graph).unwrap();
        for q in &quiz.questions {
            assert!(
                !q.explanation.is_empty(),
                "Question '{}' should have a non-empty explanation",
                q.question
            );
        }
    }

    #[test]
    fn quiz_types_varied() {
        let graph = make_test_graph_with_edges();
        let chapter = make_test_chapter();
        let quiz = generate_quiz(&chapter, &graph).unwrap();
        let kinds: Vec<QuizQuestionKind> = quiz.questions.iter().map(|q| q.kind).collect();
        // Should have at least 2 different kinds
        let unique_kinds: std::collections::HashSet<_> = kinds.iter().collect();
        assert!(
            unique_kinds.len() >= 2,
            "Quiz should have at least 2 different question types, got {:?}",
            unique_kinds
        );
    }
}
