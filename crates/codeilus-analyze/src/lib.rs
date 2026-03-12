//! Anti-pattern detection, security hotspots, test gaps, refactoring suggestions.

pub mod circular_deps;
pub mod god_class;
pub mod long_method;
pub mod security;
pub mod test_gap;
pub mod types;

pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ParsedFile;

/// Run all analyzers and return combined findings.
pub fn analyze(
    parsed_files: &[ParsedFile],
    graph: &KnowledgeGraph,
) -> CodeilusResult<Vec<PatternFinding>> {
    let mut findings = Vec::new();
    findings.extend(god_class::detect(parsed_files)?);
    findings.extend(long_method::detect(parsed_files)?);
    findings.extend(circular_deps::detect(graph)?);
    findings.extend(security::detect(parsed_files)?);
    findings.extend(test_gap::detect(parsed_files)?);
    Ok(findings)
}
