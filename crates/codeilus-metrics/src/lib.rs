//! Code metrics: SLOC, fan-in/out, complexity, modularity, TF-IDF, heatmaps.

pub mod complexity;
pub mod fan;
pub mod git;
pub mod heatmap;
pub mod modularity;
pub mod sloc;
pub mod tfidf;
pub mod types;

pub use types::*;

use std::collections::HashMap;
use std::path::Path;

use codeilus_core::ids::{FileId, SymbolId};
use codeilus_core::CodeilusResult;
use codeilus_graph::KnowledgeGraph;
use codeilus_parse::ParsedFile;

/// Compute all metrics for parsed files and graph.
pub fn compute_metrics(
    parsed_files: &[ParsedFile],
    graph: &KnowledgeGraph,
    repo_path: &Path,
) -> CodeilusResult<MetricsReport> {
    // Build file index
    let mut file_id_map: HashMap<String, FileId> = HashMap::new();
    let mut next_file_id: i64 = 1;
    for pf in parsed_files {
        let path = pf.path.to_string_lossy().to_string();
        file_id_map.insert(path, FileId(next_file_id));
        next_file_id += 1;
    }

    // 1. Fan-in/out from graph
    let fan_data = fan::compute_fan(graph);

    // 2. Complexity per symbol
    let mut symbol_sources: HashMap<SymbolId, Vec<String>> = HashMap::new();
    let mut symbol_locs: HashMap<SymbolId, usize> = HashMap::new();
    let mut symbol_id_counter: i64 = 1;

    // Map symbol names to IDs matching the graph
    let mut name_file_to_sid: HashMap<(String, String), SymbolId> = HashMap::new();
    for idx in graph.graph.node_indices() {
        let node = &graph.graph[idx];
        // We'll map by name for source extraction
        name_file_to_sid
            .entry((node.name.clone(), String::new()))
            .or_insert(node.symbol_id);
    }

    for pf in parsed_files {
        let source = std::fs::read_to_string(&pf.path).unwrap_or_default();
        let lines: Vec<&str> = source.lines().collect();

        for sym in &pf.symbols {
            let sid = graph
                .node_index
                .keys()
                .find(|id| {
                    let idx = graph.node_index[id];
                    graph.graph[idx].name == sym.name
                })
                .copied()
                .unwrap_or_else(|| {
                    let id = SymbolId(symbol_id_counter);
                    symbol_id_counter += 1;
                    id
                });

            let start = (sym.start_line - 1).max(0) as usize;
            let end = (sym.end_line as usize).min(lines.len());
            let loc = end.saturating_sub(start);
            symbol_locs.insert(sid, loc);

            if start < lines.len() {
                let sym_lines: Vec<String> = lines[start..end]
                    .iter()
                    .map(|s| s.to_string())
                    .collect();
                symbol_sources.insert(sid, sym_lines);
            }
        }
    }

    let complexity_map = complexity::compute_complexity(&symbol_sources, &symbol_locs);

    // 3. Git metrics
    let git_data = git::compute_git_metrics(repo_path, 1000);

    // 4. SLOC and file metrics
    let mut file_metrics: Vec<FileMetrics> = Vec::new();
    let mut language_breakdown_data: Vec<(codeilus_core::Language, usize)> = Vec::new();
    let mut total_sloc = 0;
    let mut total_symbols = 0;

    for pf in parsed_files {
        let path = pf.path.to_string_lossy().to_string();
        let file_id = file_id_map[&path];

        let source = std::fs::read_to_string(&pf.path).unwrap_or_default();
        let sloc_count = sloc::count_sloc(&source, pf.language);
        let sloc_actual = if sloc_count > 0 { sloc_count } else { pf.sloc };

        // File-level complexity: average of symbol complexities in this file
        let file_complexity: f64 = if pf.symbols.is_empty() {
            1.0
        } else {
            let mut sum = 0.0;
            let mut count = 0;
            for sym in &pf.symbols {
                if let Some(sid) = graph.node_index.keys().find(|id| {
                    let idx = graph.node_index[id];
                    graph.graph[idx].name == sym.name
                }) {
                    if let Some(&c) = complexity_map.get(sid) {
                        sum += c;
                        count += 1;
                    }
                }
            }
            if count > 0 { sum / count as f64 } else { 1.0 }
        };

        let (churn, contributors) = git_data.get(&path).copied().unwrap_or((0, 0));

        language_breakdown_data.push((pf.language, sloc_actual));
        total_sloc += sloc_actual;
        total_symbols += pf.symbols.len();

        file_metrics.push(FileMetrics {
            file_id,
            path,
            sloc: sloc_actual,
            complexity: file_complexity,
            churn,
            contributors,
            heatmap_score: 0.0,
        });
    }

    // 5. Heatmap scoring
    let fan_in_per_file: HashMap<FileId, usize> = {
        let mut map = HashMap::new();
        for idx in graph.graph.node_indices() {
            let node = &graph.graph[idx];
            let fi = fan_data.get(&node.symbol_id).map(|(fi, _)| *fi).unwrap_or(0);
            *map.entry(node.file_id).or_default() += fi;
        }
        map
    };
    heatmap::compute_heatmap(&mut file_metrics, &fan_in_per_file);

    // 6. Symbol metrics
    let symbol_metrics: Vec<SymbolMetrics> = fan_data
        .iter()
        .map(|(sid, (fi, fo))| {
            let loc = symbol_locs.get(sid).copied().unwrap_or(0);
            let cx = complexity_map.get(sid).copied().unwrap_or(1.0);
            SymbolMetrics {
                symbol_id: *sid,
                fan_in: *fi,
                fan_out: *fo,
                complexity: cx,
                loc,
            }
        })
        .collect();

    // 7. Modularity
    let (global_q, per_community_q) = modularity::compute_modularity(graph);

    // 8. TF-IDF per community
    let community_names: Vec<Vec<String>> = graph
        .communities
        .iter()
        .map(|c| {
            c.members
                .iter()
                .filter_map(|sid| {
                    graph.node_index.get(sid).map(|idx| graph.graph[*idx].name.clone())
                })
                .collect()
        })
        .collect();
    let tfidf_results = tfidf::compute_tfidf(&community_names, 10);

    // 9. Community metrics
    let community_metrics: Vec<CommunityMetrics> = graph
        .communities
        .iter()
        .enumerate()
        .map(|(i, c)| {
            let cm_sloc: usize = c
                .members
                .iter()
                .filter_map(|sid| symbol_locs.get(sid))
                .sum();
            let q = per_community_q.get(&c.id).copied().unwrap_or(0.0);
            let keywords = tfidf_results.get(i).cloned().unwrap_or_default();

            CommunityMetrics {
                community_id: c.id,
                modularity_q: q,
                keywords,
                total_sloc: cm_sloc,
                member_count: c.members.len(),
            }
        })
        .collect();

    // 10. Language breakdown
    let language_breakdown = sloc::language_breakdown(&language_breakdown_data);

    let avg_complexity = if !file_metrics.is_empty() {
        file_metrics.iter().map(|f| f.complexity).sum::<f64>() / file_metrics.len() as f64
    } else {
        0.0
    };

    let repo_metrics = RepoMetrics {
        total_files: parsed_files.len(),
        total_sloc,
        total_symbols,
        language_breakdown,
        avg_complexity,
        modularity_q: global_q,
    };

    Ok(MetricsReport {
        file_metrics,
        symbol_metrics,
        community_metrics,
        repo_metrics,
    })
}
