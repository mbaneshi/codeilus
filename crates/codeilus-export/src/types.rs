use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportData {
    pub repo_name: String,
    pub repo_description: Option<String>,
    pub language_badges: Vec<LanguageBadge>,
    pub overview: String,
    pub architecture_mermaid: String,
    pub reading_order: Vec<ReadingOrderEntry>,
    pub entry_points: Vec<EntryPointEntry>,
    pub architecture_narrative: String,
    pub extension_guide: String,
    pub contribution_guide: String,
    pub why_trending: String,
    pub metrics_snapshot: MetricsSnapshot,
    pub file_tree: String,
    pub communities: Vec<CommunityExport>,
    pub patterns: Vec<PatternExport>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageBadge {
    pub language: String,
    pub percentage: f64,
    pub color: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReadingOrderEntry {
    pub path: String,
    pub reason: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntryPointEntry {
    pub name: String,
    pub file_path: String,
    pub score: f64,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsSnapshot {
    pub total_files: usize,
    pub total_sloc: usize,
    pub total_symbols: usize,
    pub avg_complexity: f64,
    pub modularity_q: f64,
    pub hotspot_files: Vec<HotspotFile>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HotspotFile {
    pub path: String,
    pub heatmap_score: f64,
    pub complexity: f64,
    pub churn: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommunityExport {
    pub label: String,
    pub summary: String,
    pub member_count: usize,
    pub key_symbols: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternExport {
    pub kind: String,
    pub severity: String,
    pub message: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExportedRepo {
    pub name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub file_path: String,
    pub file_size_kb: usize,
    pub exported_at: String,
}
