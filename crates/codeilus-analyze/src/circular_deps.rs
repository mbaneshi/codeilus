use codeilus_core::error::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use petgraph::algo::tarjan_scc;

use crate::types::{PatternFinding, PatternKind, Severity};

/// Detect circular dependencies in the knowledge graph using Tarjan's SCC algorithm.
pub fn detect(graph: &KnowledgeGraph) -> CodeilusResult<Vec<PatternFinding>> {
    let mut findings = Vec::new();

    let sccs = tarjan_scc(&graph.graph);
    for scc in &sccs {
        // A single-node SCC is not a cycle (unless it has a self-edge, which we ignore)
        if scc.len() <= 1 {
            continue;
        }

        // Build the cycle description from node names
        let names: Vec<String> = scc
            .iter()
            .map(|idx| graph.graph[*idx].name.clone())
            .collect();
        let cycle_str = names.join(" → ");
        let message = format!(
            "Circular: {} → {}",
            cycle_str,
            names.first().unwrap_or(&String::new())
        );

        findings.push(PatternFinding {
            kind: PatternKind::CircularDependency,
            severity: Severity::Warning,
            file_id: None,
            symbol_id: None,
            file_path: String::new(),
            line: None,
            message,
            suggestion: "Break the cycle by extracting shared types into a common module"
                .to_string(),
        });
    }

    Ok(findings)
}
