//! Domain events broadcast through the EventBus.

use serde::{Deserialize, Serialize};

/// All events that flow through the system.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", content = "data")]
pub enum CodeilusEvent {
    // Analysis pipeline
    AnalysisStarted { path: String },
    ParsingProgress { files_done: usize, files_total: usize },
    ParsingComplete { files: usize, symbols: usize },
    GraphBuilding,
    GraphComplete { nodes: usize, edges: usize, communities: usize },
    MetricsComputed { files: usize },
    DiagramGenerated { kind: String },
    LearningPathGenerated { chapters: usize },

    // LLM
    LlmStreamChunk { text: String },
    LlmStreamComplete,

    // Narrative
    NarrativeGenerated { kind: String, target: String },
    NarrativeProgress { done: usize, total: usize },

    // Harvest
    HarvestStarted { date: String },
    HarvestRepoFound { owner: String, name: String, stars_today: u32 },
    HarvestComplete { repos: usize },

    // Export
    ExportStarted { repo: String },
    ExportComplete { path: String, size_bytes: u64 },

    // General
    Error { message: String },
}
