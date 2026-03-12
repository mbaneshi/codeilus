use codeilus_core::error::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use petgraph::algo::tarjan_scc;

use crate::types::{PatternFinding, PatternKind, Severity};

/// Detect circular dependencies in the knowledge graph using Tarjan's SCC algorithm.
pub fn detect(graph: &KnowledgeGraph) -> CodeilusResult<Vec<PatternFinding>> {
    let mut findings = Vec::new();

    let sccs = tarjan_scc(&graph.graph);

    for scc in &sccs {
        // Only report components with >1 node (actual cycles)
        if scc.len() <= 1 {
            continue;
        }

        let names: Vec<&str> = scc
            .iter()
            .map(|idx| graph.graph[*idx].name.as_str())
            .collect();

        let cycle_str = format!(
            "{} → {}",
            names.join(" → "),
            names.first().unwrap_or(&"?")
        );

        findings.push(PatternFinding {
            kind: PatternKind::CircularDependency,
            severity: Severity::Warning,
            file_id: None,
            symbol_id: None,
            file_path: String::new(),
            line: None,
            message: format!("Circular: {cycle_str}"),
            suggestion: "Break the cycle by extracting shared types into a common module"
                .to_string(),
        });
    }

    Ok(findings)
}
