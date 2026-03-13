use codeilus_core::types::NarrativeKind;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ParsedFile;
use std::collections::HashSet;

/// Generate placeholder content for a narrative kind when LLM is unavailable.
/// Uses data-driven content where possible.
pub fn placeholder_for(
    kind: NarrativeKind,
    graph: &KnowledgeGraph,
    parsed_files: &[ParsedFile],
    target_id: Option<i64>,
) -> String {
    match kind {
        NarrativeKind::Overview => overview_placeholder(parsed_files),
        NarrativeKind::Architecture => architecture_placeholder(graph),
        NarrativeKind::ReadingOrder => reading_order_placeholder(graph),
        NarrativeKind::ExtensionGuide => generic_placeholder("Extension Guide"),
        NarrativeKind::ContributionGuide => generic_placeholder("Contribution Guide"),
        NarrativeKind::WhyTrending => generic_placeholder("Why Trending"),
        NarrativeKind::ModuleSummary => module_summary_placeholder(graph, target_id),
        NarrativeKind::SymbolExplanation => symbol_explanation_placeholder(graph, target_id),
    }
}

fn overview_placeholder(parsed_files: &[ParsedFile]) -> String {
    let n = parsed_files.len();
    let languages: HashSet<String> = parsed_files
        .iter()
        .map(|f| f.language.to_string())
        .collect();
    let langs: Vec<&str> = languages.iter().map(|s| s.as_str()).collect();
    let lang_list = if langs.is_empty() {
        "unknown languages".to_string()
    } else {
        langs.join(", ")
    };
    format!(
        "This project contains {} files across {}. Run with an LLM available for a detailed overview.",
        n, lang_list
    )
}

fn architecture_placeholder(graph: &KnowledgeGraph) -> String {
    let n = graph.communities.len();
    if n == 0 {
        "No community/module structure detected yet. Run graph analysis first, then re-generate with an LLM available.".to_string()
    } else {
        let labels: Vec<&str> = graph.communities.iter().map(|c| c.label.as_str()).collect();
        format!(
            "This codebase has {} communities/modules: {}. An LLM is needed to generate architectural descriptions.",
            n,
            labels.join(", ")
        )
    }
}

fn reading_order_placeholder(graph: &KnowledgeGraph) -> String {
    if graph.entry_points.is_empty() {
        return "No entry points detected yet. Run graph analysis first.".to_string();
    }

    let mut lines = vec!["Recommended reading order (by entry point score):".to_string()];
    let mut sorted = graph.entry_points.clone();
    sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    for (i, ep) in sorted.iter().take(5).enumerate() {
        // Look up the symbol name in the graph
        let name = graph
            .node_index
            .get(&ep.symbol_id)
            .map(|idx| graph.graph[*idx].name.clone())
            .unwrap_or_else(|| format!("symbol_{}", ep.symbol_id.0));
        lines.push(format!(
            "{}. {} (score: {:.2}) — {}",
            i + 1,
            name,
            ep.score,
            ep.reason
        ));
    }
    lines.join("\n")
}

fn module_summary_placeholder(graph: &KnowledgeGraph, target_id: Option<i64>) -> String {
    let target = target_id.unwrap_or(0);
    if let Some(community) = graph.communities.iter().find(|c| c.id.0 == target) {
        let member_count = community.members.len();
        let member_names: Vec<String> = community
            .members
            .iter()
            .take(10)
            .filter_map(|sid| {
                graph
                    .node_index
                    .get(sid)
                    .map(|idx| graph.graph[*idx].name.clone())
            })
            .collect();
        let names_str = if member_names.is_empty() {
            "no named members".to_string()
        } else {
            member_names.join(", ")
        };
        format!(
            "Module '{}' contains {} symbols including: {}. Cohesion score: {:.2}.",
            community.label, member_count, names_str, community.cohesion
        )
    } else {
        format!("Module summary for community {} — no data available yet.", target)
    }
}

fn symbol_explanation_placeholder(graph: &KnowledgeGraph, target_id: Option<i64>) -> String {
    let target = target_id.unwrap_or(0);
    let sid = codeilus_core::ids::SymbolId(target);
    if let Some(idx) = graph.node_index.get(&sid) {
        let node = &graph.graph[*idx];
        format!(
            "'{}' is a {} symbol. An LLM is needed to generate a detailed explanation.",
            node.name, node.kind
        )
    } else {
        format!("Symbol {} — no data available yet.", target)
    }
}

fn generic_placeholder(section: &str) -> String {
    format!(
        "An LLM is required to generate the {}. Ensure an LLM provider is configured and available.",
        section
    )
}
