use crate::placeholders;
use crate::prompts;
use crate::types::Narrative;
use codeilus_core::types::NarrativeKind;
use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_llm::{build_context, ClaudeCli, ContextFocus, LlmRequest};
use codeilus_parse::ParsedFile;
use std::path::Path;
use tracing::{info, warn};

/// How many community summaries to request per LLM call.
const MODULE_BATCH_SIZE: usize = 5;

pub struct NarrativeGenerator {
    cli: ClaudeCli,
    llm_available: bool,
}

impl NarrativeGenerator {
    pub async fn new() -> Self {
        // Check CODEILUS_SKIP_LLM env var
        let skip_llm = std::env::var("CODEILUS_SKIP_LLM")
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

        let cli = ClaudeCli::with_timeout(180);
        let llm_available = if skip_llm {
            info!("CODEILUS_SKIP_LLM=1 — skipping all LLM calls");
            false
        } else {
            let avail = cli.is_available().await;
            if avail {
                info!("Claude CLI detected — narratives will use LLM");
            } else {
                warn!("Claude CLI not found — narratives will use placeholders");
            }
            avail
        };

        Self {
            cli,
            llm_available,
        }
    }

    /// Force placeholder mode (for testing or when LLM is not wanted).
    pub fn placeholder_only() -> Self {
        Self {
            cli: ClaudeCli::new(),
            llm_available: false,
        }
    }

    /// Generate all narratives for the repo.
    pub async fn generate_all(
        &self,
        graph: &KnowledgeGraph,
        parsed_files: &[ParsedFile],
        _repo_path: &Path,
    ) -> CodeilusResult<Vec<Narrative>> {
        let mut narratives = Vec::new();

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

        // Per-community ModuleSummary — batched when using LLM
        if self.llm_available && !graph.communities.is_empty() {
            let batched = self
                .generate_module_summaries_batched(graph, parsed_files)
                .await?;
            narratives.extend(batched);
        } else {
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
        }

        info!(
            count = narratives.len(),
            llm = self.llm_available,
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

    /// Batch community summaries: groups communities into batches of MODULE_BATCH_SIZE
    /// and asks the LLM to summarize all communities in one prompt per batch.
    async fn generate_module_summaries_batched(
        &self,
        graph: &KnowledgeGraph,
        parsed_files: &[ParsedFile],
    ) -> CodeilusResult<Vec<Narrative>> {
        let mut narratives = Vec::new();

        for batch in graph.communities.chunks(MODULE_BATCH_SIZE) {
            // Build combined context for all communities in this batch
            let mut combined_context = String::new();
            let mut community_ids: Vec<(i64, String)> = Vec::new();

            for community in batch {
                let context = build_context(graph, ContextFocus::Community(community.id.0));
                combined_context.push_str(&format!(
                    "\n---\n## Community {} (\"{}\")\n{}\n",
                    community.id.0, community.label, context
                ));
                community_ids.push((community.id.0, community.label.clone()));
            }

            let labels_list: Vec<String> = community_ids
                .iter()
                .map(|(id, label)| format!("{} (\"{}\")", id, label))
                .collect();

            let prompt_template = prompts::get_prompt(NarrativeKind::ModuleSummary);
            let user_prompt = format!(
                "Write a 1-2 paragraph summary for EACH of the following {} communities/modules. \
                 For each, use beginner-friendly analogies and explain what it does, its key types, \
                 and how it connects to the rest of the codebase.\n\n\
                 Communities: {}\n\n\
                 Format your response as:\n\
                 ## Community <id>\n<summary>\n\n\
                 Context:\n{}",
                batch.len(),
                labels_list.join(", "),
                combined_context,
            );

            let request = LlmRequest {
                prompt: user_prompt,
                system: Some(prompt_template.system.to_string()),
                max_tokens: Some(2048),
            };

            info!(
                batch_size = batch.len(),
                communities = ?labels_list,
                "generating batched module summaries via Claude CLI"
            );

            match self.cli.prompt(&request).await {
                Ok(response) => {
                    // Parse the batched response — split by "## Community <id>" headers
                    let sections = split_batched_response(&response.text, &community_ids);
                    for (id, label) in &community_ids {
                        let content = sections
                            .iter()
                            .find(|(sid, _)| sid == id)
                            .map(|(_, text)| text.clone())
                            .unwrap_or_else(|| {
                                format!(
                                    "Narrative generation failed for module '{}' — \
                                     run codeilus analyze again to retry",
                                    label
                                )
                            });

                        let title = graph
                            .communities
                            .iter()
                            .find(|c| c.id.0 == *id)
                            .map(|c| format!("Module: {}", c.label))
                            .unwrap_or_else(|| format!("Module Summary (Community {})", id));

                        narratives.push(Narrative {
                            kind: NarrativeKind::ModuleSummary,
                            target_id: Some(*id),
                            title,
                            content,
                            is_placeholder: false,
                        });
                    }
                }
                Err(e) => {
                    warn!(error = %e, "batched LLM call failed, using placeholders");
                    for (id, _) in &community_ids {
                        let content = placeholders::placeholder_for(
                            NarrativeKind::ModuleSummary,
                            graph,
                            parsed_files,
                            Some(*id),
                        );
                        let title = graph
                            .communities
                            .iter()
                            .find(|c| c.id.0 == *id)
                            .map(|c| format!("Module: {}", c.label))
                            .unwrap_or_else(|| format!("Module Summary (Community {})", id));

                        narratives.push(Narrative {
                            kind: NarrativeKind::ModuleSummary,
                            target_id: Some(*id),
                            title,
                            content,
                            is_placeholder: true,
                        });
                    }
                }
            }
        }

        Ok(narratives)
    }

    async fn generate_one_internal(
        &self,
        kind: NarrativeKind,
        graph: &KnowledgeGraph,
        parsed_files: &[ParsedFile],
        target_id: Option<i64>,
    ) -> CodeilusResult<Narrative> {
        let title = title_for(kind, target_id, graph);

        if !self.llm_available {
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

        // Build context based on narrative kind
        let focus = match kind {
            NarrativeKind::Overview
            | NarrativeKind::Architecture
            | NarrativeKind::ReadingOrder
            | NarrativeKind::ExtensionGuide
            | NarrativeKind::ContributionGuide
            | NarrativeKind::WhyTrending => ContextFocus::Overview,
            NarrativeKind::ModuleSummary => {
                if let Some(id) = target_id {
                    ContextFocus::Community(id)
                } else {
                    ContextFocus::Overview
                }
            }
            NarrativeKind::SymbolExplanation => {
                if let Some(id) = target_id {
                    ContextFocus::Symbol(id)
                } else {
                    ContextFocus::Overview
                }
            }
        };

        let context = build_context(graph, focus);
        let prompt_template = prompts::get_prompt(kind);
        let user_prompt = prompt_template.user_template.replace("{context}", &context);

        let request = LlmRequest {
            prompt: user_prompt,
            system: Some(prompt_template.system.to_string()),
            max_tokens: Some(2048),
        };

        info!(kind = ?kind, target_id, "generating narrative via Claude CLI");

        match self.cli.prompt(&request).await {
            Ok(response) => {
                info!(kind = ?kind, target_id, tokens = response.tokens_used, "narrative generated via LLM");
                Ok(Narrative {
                    kind,
                    target_id,
                    title,
                    content: response.text,
                    is_placeholder: false,
                })
            }
            Err(e) => {
                warn!(kind = ?kind, error = %e, "LLM failed, falling back to placeholder");
                Ok(Narrative {
                    kind,
                    target_id,
                    title,
                    content: "Narrative generation failed — run codeilus analyze again to retry"
                        .to_string(),
                    is_placeholder: true,
                })
            }
        }
    }
}

/// Split a batched LLM response into per-community sections.
/// Looks for "## Community <id>" headers in the response text.
fn split_batched_response(text: &str, community_ids: &[(i64, String)]) -> Vec<(i64, String)> {
    let mut results = Vec::new();

    for (i, (id, _)) in community_ids.iter().enumerate() {
        let header = format!("## Community {}", id);
        if let Some(start) = text.find(&header) {
            let content_start = start + header.len();
            // Find the next community header or end of text
            let end = if i + 1 < community_ids.len() {
                let next_header = format!("## Community {}", community_ids[i + 1].0);
                text[content_start..]
                    .find(&next_header)
                    .map(|pos| content_start + pos)
                    .unwrap_or(text.len())
            } else {
                text.len()
            };
            let content = text[content_start..end].trim().to_string();
            results.push((*id, content));
        }
    }

    results
}

fn title_for(kind: NarrativeKind, target_id: Option<i64>, graph: &KnowledgeGraph) -> String {
    match kind {
        NarrativeKind::Overview => "Project Overview".to_string(),
        NarrativeKind::Architecture => "Architecture".to_string(),
        NarrativeKind::ReadingOrder => "Recommended Reading Order".to_string(),
        NarrativeKind::ExtensionGuide => "Extension Guide".to_string(),
        NarrativeKind::ContributionGuide => "Contribution Guide".to_string(),
        NarrativeKind::WhyTrending => "Why This Project Is Trending".to_string(),
        NarrativeKind::ModuleSummary => {
            if let Some(id) = target_id {
                // Try to find community label
                graph
                    .communities
                    .iter()
                    .find(|c| c.id.0 == id)
                    .map(|c| format!("Module: {}", c.label))
                    .unwrap_or_else(|| format!("Module Summary (Community {})", id))
            } else {
                "Module Summary".to_string()
            }
        }
        NarrativeKind::SymbolExplanation => {
            if let Some(id) = target_id {
                use codeilus_core::ids::SymbolId;
                graph
                    .node_index
                    .get(&SymbolId(id))
                    .map(|&idx| graph.graph[idx].name.clone())
                    .unwrap_or_else(|| format!("Symbol {}", id))
            } else {
                "Symbol Explanation".to_string()
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn split_batched_response_basic() {
        let text = "## Community 1\nFirst module does X.\n\n## Community 2\nSecond module does Y.";
        let ids = vec![(1, "core".to_string()), (2, "db".to_string())];
        let result = split_batched_response(text, &ids);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].0, 1);
        assert!(result[0].1.contains("First module"));
        assert_eq!(result[1].0, 2);
        assert!(result[1].1.contains("Second module"));
    }

    #[test]
    fn split_batched_response_missing_section() {
        let text = "## Community 1\nOnly one here.";
        let ids = vec![(1, "core".to_string()), (2, "db".to_string())];
        let result = split_batched_response(text, &ids);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0].0, 1);
    }
}
