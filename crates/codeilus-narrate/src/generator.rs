use crate::placeholders;
use crate::types::Narrative;
use codeilus_core::types::NarrativeKind;
use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ParsedFile;
use std::path::Path;
use tracing::info;

pub struct NarrativeGenerator {
    /// When true, always use placeholders (LLM unavailable).
    placeholder_mode: bool,
}

impl Default for NarrativeGenerator {
    fn default() -> Self {
        Self::new()
    }
}

impl NarrativeGenerator {
    pub fn new() -> Self {
        // Default to placeholder mode since codeilus-llm is not wired yet.
        Self {
            placeholder_mode: true,
        }
    }

    /// Generate all narratives for the repo.
    /// Order: Overview, Architecture, ReadingOrder, ExtensionGuide,
    ///        ContributionGuide, WhyTrending, then per-community ModuleSummary.
    /// SymbolExplanation is on-demand only (not pre-generated for all symbols).
    pub async fn generate_all(
        &self,
        graph: &KnowledgeGraph,
        parsed_files: &[ParsedFile],
        _repo_path: &Path,
    ) -> CodeilusResult<Vec<Narrative>> {
        let mut narratives = Vec::new();

        // Global narratives (no target_id)
        let global_kinds = [
            NarrativeKind::Overview,
            NarrativeKind::Architecture,
            NarrativeKind::ReadingOrder,
            NarrativeKind::ExtensionGuide,
            NarrativeKind::ContributionGuide,
            NarrativeKind::WhyTrending,
        ];

        for kind in &global_kinds {
            let narrative = self
                .generate_one_internal(*kind, graph, parsed_files, None)
                .await?;
            narratives.push(narrative);
        }

        // Per-community ModuleSummary
        for community in &graph.communities {
            let narrative = self
                .generate_one_internal(
                    NarrativeKind::ModuleSummary,
                    graph,
                    parsed_files,
                    Some(community.id.0),
                )
                .await?;
            narratives.push(narrative);
        }

        info!(
            count = narratives.len(),
            placeholder = self.placeholder_mode,
            "generated all narratives"
        );

        Ok(narratives)
    }

    /// Generate a single narrative.
    pub async fn generate_one(
        &self,
        kind: NarrativeKind,
        graph: &KnowledgeGraph,
        target_id: Option<i64>,
    ) -> CodeilusResult<Narrative> {
        self.generate_one_internal(kind, graph, &[], target_id)
            .await
    }

    /// Generate on-demand symbol explanation.
    pub async fn explain_symbol(
        &self,
        symbol_id: i64,
        graph: &KnowledgeGraph,
    ) -> CodeilusResult<Narrative> {
        self.generate_one_internal(
            NarrativeKind::SymbolExplanation,
            graph,
            &[],
            Some(symbol_id),
        )
        .await
    }

    async fn generate_one_internal(
        &self,
        kind: NarrativeKind,
        graph: &KnowledgeGraph,
        parsed_files: &[ParsedFile],
        target_id: Option<i64>,
    ) -> CodeilusResult<Narrative> {
        let title = title_for(kind, target_id);

        if self.placeholder_mode {
            let content = placeholders::placeholder_for(kind, graph, parsed_files, target_id);
            info!(kind = ?kind, target_id, "generated placeholder narrative");
            return Ok(Narrative {
                kind,
                target_id,
                title,
                content,
                is_placeholder: true,
            });
        }

        // Future: LLM integration will go here.
        // For now, always fall through to placeholder.
        let content = placeholders::placeholder_for(kind, graph, parsed_files, target_id);
        Ok(Narrative {
            kind,
            target_id,
            title,
            content,
            is_placeholder: true,
        })
    }
}

fn title_for(kind: NarrativeKind, target_id: Option<i64>) -> String {
    match kind {
        NarrativeKind::Overview => "Project Overview".to_string(),
        NarrativeKind::Architecture => "Architecture".to_string(),
        NarrativeKind::ReadingOrder => "Recommended Reading Order".to_string(),
        NarrativeKind::ExtensionGuide => "Extension Guide".to_string(),
        NarrativeKind::ContributionGuide => "Contribution Guide".to_string(),
        NarrativeKind::WhyTrending => "Why This Project Is Trending".to_string(),
        NarrativeKind::ModuleSummary => {
            if let Some(id) = target_id {
                format!("Module Summary (Community {})", id)
            } else {
                "Module Summary".to_string()
            }
        }
        NarrativeKind::SymbolExplanation => {
            if let Some(id) = target_id {
                format!("Symbol Explanation (Symbol {})", id)
            } else {
                "Symbol Explanation".to_string()
            }
        }
    }
}
