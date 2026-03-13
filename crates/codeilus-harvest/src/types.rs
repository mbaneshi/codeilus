use serde::{Deserialize, Serialize};

/// Configuration for the trending harvest.
#[derive(Debug, Clone)]
pub struct HarvestConfig {
    /// Filter by programming language (e.g., "rust", "python").
    pub language: Option<String>,
    /// Time range for trending.
    pub since: TrendingSince,
    /// Maximum number of repos to harvest (default 25).
    pub max_repos: usize,
    /// Directory for shallow clones.
    pub clone_dir: std::path::PathBuf,
    /// Maximum parallel clones (default 5).
    pub concurrent_clones: usize,
}

impl Default for HarvestConfig {
    fn default() -> Self {
        Self {
            language: None,
            since: TrendingSince::Daily,
            max_repos: 25,
            clone_dir: std::path::PathBuf::from("./clones"),
            concurrent_clones: 5,
        }
    }
}

/// Time range for trending repos.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrendingSince {
    Daily,
    Weekly,
    Monthly,
}

impl TrendingSince {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Daily => "daily",
            Self::Weekly => "weekly",
            Self::Monthly => "monthly",
        }
    }
}

/// A repository discovered from GitHub trending.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HarvestedRepo {
    pub owner: String,
    pub name: String,
    pub full_name: String,
    pub description: Option<String>,
    pub language: Option<String>,
    pub stars_today: Option<usize>,
    pub total_stars: Option<usize>,
    pub url: String,
    pub clone_url: String,
    pub fingerprint: String,
    pub status: HarvestStatus,
}

/// Lifecycle status of a harvested repo.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HarvestStatus {
    Found,
    Cloning,
    Cloned,
    Analyzing,
    Complete,
    Failed,
    Skipped,
}

impl HarvestStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Found => "found",
            Self::Cloning => "cloning",
            Self::Cloned => "cloned",
            Self::Analyzing => "analyzing",
            Self::Complete => "complete",
            Self::Failed => "failed",
            Self::Skipped => "skipped",
        }
    }

    pub fn parse(s: &str) -> Self {
        match s {
            "found" => Self::Found,
            "cloning" => Self::Cloning,
            "cloned" => Self::Cloned,
            "analyzing" => Self::Analyzing,
            "complete" => Self::Complete,
            "failed" => Self::Failed,
            "skipped" => Self::Skipped,
            _ => Self::Found,
        }
    }
}
