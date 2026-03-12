//! Codeilus error hierarchy and result type.

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CodeilusError {
    #[error("Database error: {0}")]
    Database(#[from] Box<dyn std::error::Error + Send + Sync>),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Parse error: {0}")]
    Parse(String),

    #[error("Graph error: {0}")]
    Graph(String),

    #[error("LLM error: {0}")]
    Llm(String),

    #[error("Harvest error: {0}")]
    Harvest(String),

    #[error("Export error: {0}")]
    Export(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Event bus error: {0}")]
    EventBus(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("{0}")]
    Internal(String),
}

pub type CodeilusResult<T> = Result<T, CodeilusError>;
