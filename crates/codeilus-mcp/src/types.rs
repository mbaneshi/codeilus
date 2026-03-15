//! MCP tool input types.

use schemars::JsonSchema;
use serde::Deserialize;

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QuerySymbolsInput {
    /// Search query for symbol name.
    pub query: String,
    /// Filter by symbol kind (e.g. "function", "class", "struct").
    pub kind: Option<String>,
    /// Maximum results to return (default 20).
    pub limit: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct QueryGraphInput {
    /// If provided, return neighbors of this symbol only.
    pub symbol_id: Option<i64>,
    /// BFS depth for neighbor search (default 1).
    pub depth: Option<usize>,
    /// Filter by edge kind (e.g. "CALLS", "IMPORTS").
    pub edge_kind: Option<String>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetContextInput {
    /// Focus: "overview", "symbol", "community", or "files".
    pub focus: String,
    /// Target symbol_id or community_id (for symbol/community focus).
    pub target_id: Option<i64>,
    /// File paths (for files focus).
    pub file_paths: Option<Vec<String>>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetImpactInput {
    /// Symbol to analyze blast radius for.
    pub symbol_id: i64,
    /// Blast radius depth (default 3).
    pub depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetDiagramInput {
    /// Diagram kind: "architecture" or "flowchart".
    pub kind: String,
    /// Required for flowchart — the symbol to generate control flow for.
    pub symbol_id: Option<i64>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetMetricsInput {
    /// Specific file ID, or omit for all.
    pub file_id: Option<i64>,
    /// Return top N hotspot files.
    pub top_hotspots: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetLearningStatusInput {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainSymbolInput {
    /// Symbol ID to explain.
    pub symbol_id: i64,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct UnderstandCodebaseInput {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct TraceCallChainInput {
    /// Symbol name to trace calls from.
    pub symbol_name: String,
    /// Maximum call chain depth (default 5).
    pub max_depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ImpactAnalysisInput {
    /// Symbol name to analyze.
    pub symbol_name: String,
    /// Blast radius depth 1-3 (default 2).
    pub depth: Option<usize>,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindRelatedCodeInput {
    /// Symbol name to find related code for.
    pub symbol_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct ExplainFileInput {
    /// File path to explain.
    pub file_path: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct FindTestsForInput {
    /// Symbol name to find tests for.
    pub symbol_name: String,
}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct SuggestReadingOrderInput {}

#[derive(Debug, Deserialize, JsonSchema)]
pub struct GetCommunityContextInput {
    /// Community ID.
    pub community_id: i64,
}
