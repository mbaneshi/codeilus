use codeilus_core::ids::{CommunityId, FileId, SymbolId};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsReport {
    pub file_metrics: Vec<FileMetrics>,
    pub symbol_metrics: Vec<SymbolMetrics>,
    pub community_metrics: Vec<CommunityMetrics>,
    pub repo_metrics: RepoMetrics,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileMetrics {
    pub file_id: FileId,
    pub path: String,
    pub sloc: usize,
    pub complexity: f64,
    pub churn: usize,
    pub contributors: usize,
    pub heatmap_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolMetrics {
    pub symbol_id: SymbolId,
    pub fan_in: usize,
    pub fan_out: usize,
    pub complexity: f64,
    pub loc: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityMetrics {
    pub community_id: CommunityId,
    pub modularity_q: f64,
    pub keywords: Vec<(String, f64)>,
    pub total_sloc: usize,
    pub member_count: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepoMetrics {
    pub total_files: usize,
    pub total_sloc: usize,
    pub total_symbols: usize,
    pub language_breakdown: HashMap<String, usize>,
    pub avg_complexity: f64,
    pub modularity_q: f64,
}
