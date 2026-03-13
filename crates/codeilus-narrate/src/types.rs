use codeilus_core::types::NarrativeKind;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Narrative {
    pub kind: NarrativeKind,
    pub target_id: Option<i64>,
    pub title: String,
    pub content: String,
    pub is_placeholder: bool,
}

/// Convert a NarrativeKind to its string key for DB storage.
pub fn narrative_kind_key(kind: NarrativeKind) -> &'static str {
    match kind {
        NarrativeKind::Overview => "overview",
        NarrativeKind::Architecture => "architecture",
        NarrativeKind::ExtensionGuide => "extension_guide",
        NarrativeKind::ContributionGuide => "contribution_guide",
        NarrativeKind::WhyTrending => "why_trending",
        NarrativeKind::ModuleSummary => "module_summary",
        NarrativeKind::SymbolExplanation => "symbol_explanation",
        NarrativeKind::ReadingOrder => "reading_order",
    }
}
