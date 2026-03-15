//! Pre-generate narrative content: overview, architecture, extension/contribution guides.

pub mod generator;
pub mod placeholders;
pub mod prompts;
pub mod types;

pub use generator::NarrativeGenerator;
pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_llm::LlmProvider;
use codeilus_parse::ParsedFile;
use std::path::Path;
use std::sync::Arc;

pub async fn generate_all_narratives(
    graph: &KnowledgeGraph,
    parsed_files: &[ParsedFile],
    repo_path: &Path,
    llm: Arc<dyn LlmProvider>,
) -> CodeilusResult<Vec<Narrative>> {
    NarrativeGenerator::new(llm)
        .await
        .generate_all(graph, parsed_files, repo_path)
        .await
}
