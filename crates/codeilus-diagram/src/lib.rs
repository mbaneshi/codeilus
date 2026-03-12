//! Mermaid/flowchart generation, ASCII file tree, and diagram validation.

pub mod architecture;
pub mod file_tree;
pub mod flowchart;
pub mod mermaid;
pub mod types;

pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::Symbol;

/// Generate an architecture Mermaid diagram from the knowledge graph.
pub fn generate_architecture(graph: &KnowledgeGraph) -> CodeilusResult<String> {
    architecture::generate(graph)
}

/// Generate a flowchart IR from a symbol and its source code,
/// then convert to Mermaid syntax.
pub fn generate_flowchart(symbol: &Symbol, source: &str) -> CodeilusResult<String> {
    let ir = flowchart::generate(symbol, source)?;
    Ok(flowchart::ir_to_mermaid(&ir))
}

/// Generate an ASCII file tree from sorted file paths.
pub fn generate_file_tree(files: &[String], style: TreeStyle) -> String {
    file_tree::generate(files, style)
}

/// Validate Mermaid diagram syntax.
pub fn validate_mermaid(mermaid: &str) -> ValidationResult {
    mermaid::validate(mermaid)
}
