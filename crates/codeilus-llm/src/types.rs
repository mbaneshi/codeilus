use serde::{Deserialize, Serialize};

/// A request to the LLM.
#[derive(Debug, Clone)]
pub struct LlmRequest {
    pub prompt: String,
    pub system: Option<String>,
    pub max_tokens: Option<usize>,
}

/// A complete LLM response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LlmResponse {
    pub text: String,
    pub tokens_used: usize,
}

/// Streaming events from the LLM.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LlmEvent {
    /// Partial text content received.
    ContentDelta(String),
    /// Tool usage by the agent (for streaming UX).
    ToolUse { name: String, input: String },
    /// Final result.
    Complete(LlmResponse),
    /// Error during streaming.
    Error(String),
}

/// What to focus the context on.
#[derive(Debug, Clone)]
pub enum ContextFocus {
    /// Overall repo overview.
    Overview,
    /// Specific community by ID.
    Community(i64),
    /// Specific symbol and its neighbors.
    Symbol(i64),
    /// Custom file paths.
    Files(Vec<String>),
}
