use codeilus_core::types::NarrativeKind;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ParsedFile;
use std::collections::{HashMap, HashSet};

/// Generate placeholder content for a narrative kind when LLM is unavailable.
/// Produces meaningful, data-driven content from the knowledge graph.
pub fn placeholder_for(
    kind: NarrativeKind,
    graph: &KnowledgeGraph,
    parsed_files: &[ParsedFile],
    target_id: Option<i64>,
) -> String {
    match kind {
        NarrativeKind::Overview => overview_placeholder(graph, parsed_files),
        NarrativeKind::Architecture => architecture_placeholder(graph),
        NarrativeKind::ReadingOrder => reading_order_placeholder(graph),
        NarrativeKind::ExtensionGuide => extension_guide_placeholder(graph),
        NarrativeKind::ContributionGuide => contribution_guide_placeholder(graph),
        NarrativeKind::WhyTrending => why_trending_placeholder(parsed_files),
        NarrativeKind::ModuleSummary => module_summary_placeholder(graph, target_id),
        NarrativeKind::SymbolExplanation => symbol_explanation_placeholder(graph, target_id),
    }
}

fn overview_placeholder(graph: &KnowledgeGraph, parsed_files: &[ParsedFile]) -> String {
    let n = parsed_files.len();
    let total_sloc: usize = parsed_files.iter().map(|f| f.sloc).sum();
    let total_symbols: usize = parsed_files.iter().map(|f| f.symbols.len()).sum();
    let languages: HashSet<String> = parsed_files
        .iter()
        .map(|f| f.language.to_string())
        .collect();
    let lang_list = if languages.is_empty() {
        "unknown languages".to_string()
    } else {
        let mut langs: Vec<&str> = languages.iter().map(|s| s.as_str()).collect();
        langs.sort();
        langs.join(", ")
    };

    let community_count = graph.communities.len();
    let entry_count = graph.entry_points.len();

    let mut lines = vec![
        format!("## Project Overview"),
        String::new(),
        format!(
            "This project contains **{} files** with **{} symbols** across **{} lines of code**, written in **{}**.",
            n, total_symbols, total_sloc, lang_list
        ),
        String::new(),
        format!(
            "The codebase is organized into **{} functional modules** (automatically detected communities) with **{} entry points**.",
            community_count, entry_count
        ),
    ];

    if !graph.communities.is_empty() {
        lines.push(String::new());
        lines.push("### Modules at a Glance".to_string());
        lines.push(String::new());
        for comm in &graph.communities {
            if comm.members.len() > 1 {
                lines.push(format!(
                    "- **{}** — {} symbols (cohesion: {:.0}%)",
                    format_label(&comm.label),
                    comm.members.len(),
                    comm.cohesion * 100.0
                ));
            }
        }
    }

    if !graph.entry_points.is_empty() {
        lines.push(String::new());
        lines.push("### Key Entry Points".to_string());
        lines.push(String::new());
        let mut sorted = graph.entry_points.clone();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        for ep in sorted.iter().take(5) {
            let name = graph
                .node_index
                .get(&ep.symbol_id)
                .map(|idx| graph.graph[*idx].name.clone())
                .unwrap_or_else(|| format!("symbol_{}", ep.symbol_id.0));
            lines.push(format!("- `{}` — {}", name, ep.reason));
        }
    }

    lines.join("\n")
}

fn architecture_placeholder(graph: &KnowledgeGraph) -> String {
    if graph.communities.is_empty() {
        return "No community/module structure detected yet.".to_string();
    }

    let edge_count = graph.graph.edge_count();
    let node_count = graph.graph.node_count();

    let mut lines = vec![
        "## Architecture".to_string(),
        String::new(),
        format!(
            "The codebase consists of **{} symbols** connected by **{} relationships** across **{} modules**.",
            node_count, edge_count, graph.communities.len()
        ),
        String::new(),
    ];

    // Count edge types
    let mut edge_types: HashMap<&str, usize> = HashMap::new();
    for edge_idx in graph.graph.edge_indices() {
        let edge = &graph.graph[edge_idx];
        let kind_str = match edge.kind {
            codeilus_core::types::EdgeKind::Calls => "function calls",
            codeilus_core::types::EdgeKind::Imports => "import dependencies",
            codeilus_core::types::EdgeKind::Extends => "inheritance relationships",
            codeilus_core::types::EdgeKind::Implements => "interface implementations",
            codeilus_core::types::EdgeKind::Contains => "containment relationships",
        };
        *edge_types.entry(kind_str).or_insert(0) += 1;
    }

    if !edge_types.is_empty() {
        lines.push("### Relationship Types".to_string());
        lines.push(String::new());
        let mut sorted: Vec<_> = edge_types.into_iter().collect();
        sorted.sort_by(|a, b| b.1.cmp(&a.1));
        for (kind, count) in sorted {
            lines.push(format!("- **{}** {}", count, kind));
        }
        lines.push(String::new());
    }

    // Module hierarchy
    lines.push("### Module Hierarchy".to_string());
    lines.push(String::new());
    let mut sorted_comms: Vec<_> = graph.communities.iter().filter(|c| c.members.len() > 1).collect();
    sorted_comms.sort_by(|a, b| b.members.len().cmp(&a.members.len()));
    for comm in sorted_comms.iter().take(10) {
        let top_members: Vec<String> = comm
            .members
            .iter()
            .take(3)
            .filter_map(|sid| graph.node_index.get(sid).map(|idx| graph.graph[*idx].name.clone()))
            .collect();
        lines.push(format!(
            "- **{}** ({} symbols) — includes: `{}`",
            format_label(&comm.label),
            comm.members.len(),
            top_members.join("`, `")
        ));
    }

    lines.join("\n")
}

fn reading_order_placeholder(graph: &KnowledgeGraph) -> String {
    if graph.entry_points.is_empty() {
        return "No entry points detected. Run analysis to generate reading order.".to_string();
    }

    let mut lines = vec![
        "## Recommended Reading Order".to_string(),
        String::new(),
        "Start with the entry points below, then follow the dependency chains to understand how the codebase fits together.".to_string(),
        String::new(),
    ];

    let mut sorted = graph.entry_points.clone();
    sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

    for (i, ep) in sorted.iter().take(8).enumerate() {
        let name = graph
            .node_index
            .get(&ep.symbol_id)
            .map(|idx| graph.graph[*idx].name.clone())
            .unwrap_or_else(|| format!("symbol_{}", ep.symbol_id.0));

        let kind = graph
            .node_index
            .get(&ep.symbol_id)
            .map(|idx| graph.graph[*idx].kind.clone())
            .unwrap_or_default();

        lines.push(format!(
            "{}. **`{}`** ({}) — {} *(score: {:.1})*",
            i + 1,
            name,
            kind,
            ep.reason,
            ep.score
        ));
    }

    lines.join("\n")
}

fn extension_guide_placeholder(graph: &KnowledgeGraph) -> String {
    let mut lines = vec![
        "## Extension Guide".to_string(),
        String::new(),
        "To extend this codebase, look at the modules with the highest cohesion — they represent well-defined functional areas where new features can be added cleanly.".to_string(),
        String::new(),
    ];

    // Find high fan-in symbols (good extension points)
    let mut fan_in: HashMap<codeilus_core::ids::SymbolId, usize> = HashMap::new();
    for edge_idx in graph.graph.edge_indices() {
        let (_, target) = graph.graph.edge_endpoints(edge_idx).unwrap();
        let target_node = &graph.graph[target];
        *fan_in.entry(target_node.symbol_id).or_insert(0) += 1;
    }

    let mut high_fan_in: Vec<_> = fan_in.iter().filter(|(_, &count)| count >= 3).collect();
    high_fan_in.sort_by(|a, b| b.1.cmp(a.1));

    if !high_fan_in.is_empty() {
        lines.push("### Key Extension Points (high fan-in)".to_string());
        lines.push(String::new());
        for (sid, count) in high_fan_in.iter().take(8) {
            if let Some(idx) = graph.node_index.get(sid) {
                let node = &graph.graph[*idx];
                lines.push(format!(
                    "- `{}` ({}) — used by {} other symbols",
                    node.name, node.kind, count
                ));
            }
        }
    }

    lines.join("\n")
}

fn contribution_guide_placeholder(graph: &KnowledgeGraph) -> String {
    let mut lines = vec![
        "## Contribution Guide".to_string(),
        String::new(),
        "Here's how to get started contributing to this codebase:".to_string(),
        String::new(),
        "1. **Start with entry points** — understand the main execution paths".to_string(),
        "2. **Pick a module** — each module is relatively self-contained".to_string(),
        "3. **Follow the edges** — understand how your module connects to others".to_string(),
        String::new(),
    ];

    // Find modules with lowest cohesion (may need refactoring = good contribution area)
    let mut sorted_comms: Vec<_> = graph.communities.iter().filter(|c| c.members.len() > 2).collect();
    sorted_comms.sort_by(|a, b| a.cohesion.partial_cmp(&b.cohesion).unwrap_or(std::cmp::Ordering::Equal));

    if !sorted_comms.is_empty() {
        lines.push("### Good First Areas".to_string());
        lines.push(String::new());
        for comm in sorted_comms.iter().take(5) {
            lines.push(format!(
                "- **{}** — {} symbols, cohesion {:.0}%",
                format_label(&comm.label),
                comm.members.len(),
                comm.cohesion * 100.0
            ));
        }
    }

    lines.join("\n")
}

fn why_trending_placeholder(parsed_files: &[ParsedFile]) -> String {
    let languages: HashSet<String> = parsed_files
        .iter()
        .map(|f| f.language.to_string())
        .collect();
    let lang_list = if languages.is_empty() {
        "various languages".to_string()
    } else {
        let mut langs: Vec<&str> = languages.iter().map(|s| s.as_str()).collect();
        langs.sort();
        langs.join(", ")
    };

    format!(
        "## Why This Project\n\n\
         This project is written in {} with {} files. \
         Explore the modules, architecture, and entry points to understand its design.",
        lang_list,
        parsed_files.len()
    )
}

fn module_summary_placeholder(graph: &KnowledgeGraph, target_id: Option<i64>) -> String {
    let target = target_id.unwrap_or(0);
    if let Some(community) = graph.communities.iter().find(|c| c.id.0 == target) {
        let member_count = community.members.len();

        // Gather member info
        let mut functions = Vec::new();
        let mut structs = Vec::new();
        let mut other = Vec::new();
        for sid in &community.members {
            if let Some(idx) = graph.node_index.get(sid) {
                let node = &graph.graph[*idx];
                match node.kind.to_lowercase().as_str() {
                    "function" | "method" | "fn" => functions.push(node.name.clone()),
                    "struct" | "class" | "enum" | "trait" | "interface" => structs.push(node.name.clone()),
                    _ => other.push(node.name.clone()),
                }
            }
        }

        // Count connections to other modules
        let mut external_connections = 0;
        let member_set: HashSet<_> = community.members.iter().collect();
        for sid in &community.members {
            if let Some(idx) = graph.node_index.get(sid) {
                for neighbor in graph.graph.neighbors(*idx) {
                    let neighbor_node = &graph.graph[neighbor];
                    if !member_set.contains(&neighbor_node.symbol_id) {
                        external_connections += 1;
                    }
                }
            }
        }

        let mut lines = vec![format!(
            "## Module: {}\n\nThis module contains **{} symbols** with a cohesion score of **{:.0}%** and **{} external connections**.",
            format_label(&community.label),
            member_count,
            community.cohesion * 100.0,
            external_connections
        )];

        if !structs.is_empty() {
            lines.push(format!(
                "\n### Types\n\n{}",
                structs.iter().map(|s| format!("- `{}`", s)).collect::<Vec<_>>().join("\n")
            ));
        }

        if !functions.is_empty() {
            let display_fns: Vec<_> = functions.iter().take(10).collect();
            let suffix = if functions.len() > 10 {
                format!("\n- ...and {} more", functions.len() - 10)
            } else {
                String::new()
            };
            lines.push(format!(
                "\n### Functions\n\n{}{}",
                display_fns.iter().map(|s| format!("- `{}`", s)).collect::<Vec<_>>().join("\n"),
                suffix
            ));
        }

        lines.join("\n")
    } else {
        format!("Module {} — no data available.", target)
    }
}

fn symbol_explanation_placeholder(graph: &KnowledgeGraph, target_id: Option<i64>) -> String {
    let target = target_id.unwrap_or(0);
    let sid = codeilus_core::ids::SymbolId(target);
    if let Some(idx) = graph.node_index.get(&sid) {
        let node = &graph.graph[*idx];

        // Find connections
        let callees: Vec<String> = graph
            .graph
            .neighbors(*idx)
            .map(|n| graph.graph[n].name.clone())
            .take(5)
            .collect();

        let callers: Vec<String> = graph
            .graph
            .neighbors_directed(*idx, petgraph::Direction::Incoming)
            .map(|n| graph.graph[n].name.clone())
            .take(5)
            .collect();

        let community_label = node.community_id.and_then(|cid| {
            graph.communities.iter().find(|c| c.id == cid).map(|c| format_label(&c.label))
        });

        let mut lines = vec![format!(
            "## `{}`\n\n**Kind:** {} | **Module:** {}",
            node.name,
            node.kind,
            community_label.unwrap_or_else(|| "uncategorized".to_string())
        )];

        if !callees.is_empty() {
            lines.push(format!(
                "\n**Calls:** {}",
                callees.iter().map(|s| format!("`{}`", s)).collect::<Vec<_>>().join(", ")
            ));
        }

        if !callers.is_empty() {
            lines.push(format!(
                "\n**Called by:** {}",
                callers.iter().map(|s| format!("`{}`", s)).collect::<Vec<_>>().join(", ")
            ));
        }

        lines.join("\n")
    } else {
        format!("Symbol {} — not found in the knowledge graph.", target)
    }
}

fn format_label(label: &str) -> String {
    label
        .replace("cluster_", "")
        .replace('_', " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}
