use serde::{Deserialize, Serialize};

/// Flowchart intermediate representation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowchartIR {
    pub nodes: Vec<FlowNode>,
    pub edges: Vec<FlowEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowNode {
    pub id: String,
    pub kind: FlowNodeKind,
    pub label: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FlowNodeKind {
    Entry,
    Exit,
    Process,
    Decision,
    Loop,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlowEdge {
    pub from: String,
    pub to: String,
    pub label: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TreeStyle {
    /// `├── file.rs`
    Default,
    /// `| file.rs`
    Compact,
    /// `├── file.rs (123 lines, 5 symbols)`
    Extended,
    /// `file.rs`
    Minimal,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
}
