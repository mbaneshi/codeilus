//! Learning engine: curriculum, chapters, progress tracking, gamification, quizzes.

pub mod curriculum;
pub mod progress;
pub mod quiz;
pub mod types;

pub use curriculum::generate;
pub use progress::ProgressTracker;
pub use quiz::generate_quiz;
pub use types::*;

use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;

pub fn generate_curriculum(graph: &KnowledgeGraph) -> CodeilusResult<Curriculum> {
    curriculum::generate(graph)
}
