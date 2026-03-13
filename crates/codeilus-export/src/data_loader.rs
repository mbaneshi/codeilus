use crate::types::{
    CommunityExport, ExportData, HotspotFile, LanguageBadge, MetricsSnapshot, PatternExport,
};
use codeilus_core::error::CodeilusResult;
use codeilus_db::DbPool;
use std::collections::HashMap;

/// Assign a color to a language for badge rendering.
fn language_color(lang: &str) -> &'static str {
    match lang.to_lowercase().as_str() {
        "rust" => "#dea584",
        "python" => "#3572A5",
        "typescript" => "#3178c6",
        "javascript" => "#f1e05a",
        "go" => "#00ADD8",
        "java" => "#b07219",
        "c" => "#555555",
        "cpp" => "#f34b7d",
        "csharp" => "#178600",
        "ruby" => "#701516",
        "php" => "#4F5D95",
        "swift" => "#F05138",
        "kotlin" => "#A97BFF",
        _ => "#8b949e",
    }
}

/// Load all data needed for export from the database.
pub fn load_export_data(repo_name: &str, db: &DbPool) -> CodeilusResult<ExportData> {
    let conn_arc = db.conn_arc();

    // Load files
    let file_repo = codeilus_db::FileRepo::new(conn_arc.clone());
    let files = file_repo.list(None)?;

    // Load symbols
    let symbol_repo = codeilus_db::SymbolRepo::new(conn_arc.clone());
    let total_symbols = symbol_repo.count()?;

    // Load narratives
    let narrative_repo = codeilus_db::NarrativeRepo::new(conn_arc.clone());
    let overview = narrative_repo
        .get_by_kind("overview")?
        .map(|n| n.content)
        .unwrap_or_default();
    let architecture_narrative = narrative_repo
        .get_by_kind("architecture")?
        .map(|n| n.content)
        .unwrap_or_default();
    let reading_order_text = narrative_repo
        .get_by_kind("reading_order")?
        .map(|n| n.content)
        .unwrap_or_default();
    let extension_guide = narrative_repo
        .get_by_kind("extension_guide")?
        .map(|n| n.content)
        .unwrap_or_default();
    let contribution_guide = narrative_repo
        .get_by_kind("contribution_guide")?
        .map(|n| n.content)
        .unwrap_or_default();
    let why_trending = narrative_repo
        .get_by_kind("why_trending")?
        .map(|n| n.content)
        .unwrap_or_default();

    // Parse reading order from narrative text (one entry per line, format: "N. path — reason")
    let reading_order = reading_order_text
        .lines()
        .filter(|l| l.contains('.') && l.contains('—'))
        .filter_map(|line| {
            let after_dot = line.split_once('.')?.1.trim();
            let (path_part, reason) = after_dot.split_once('—')?;
            Some(crate::types::ReadingOrderEntry {
                path: path_part.trim().to_string(),
                reason: reason.trim().to_string(),
                language: String::new(),
            })
        })
        .collect();

    // Load communities
    let community_repo = codeilus_db::CommunityRepo::new(conn_arc.clone());
    let db_communities = community_repo.list()?;
    let module_summaries = narrative_repo.list_by_kind("module_summary")?;

    let mut communities = Vec::new();
    for comm in &db_communities {
        let members = community_repo.list_members(comm.id)?;
        let member_count = members.len();

        // Look up member symbol names
        let mut key_symbols = Vec::new();
        for sid in members.iter().take(5) {
            if let Ok(sym) = symbol_repo.get(*sid) {
                key_symbols.push(sym.name);
            }
        }

        let summary = module_summaries
            .iter()
            .find(|n| n.target_id == Some(comm.id.0))
            .map(|n| n.content.clone())
            .unwrap_or_default();

        communities.push(CommunityExport {
            label: comm.label.clone(),
            summary,
            member_count,
            key_symbols,
        });
    }

    // Load patterns
    let pattern_repo = codeilus_db::PatternRepo::new(conn_arc.clone());
    let db_patterns = pattern_repo.list()?;
    let patterns: Vec<PatternExport> = db_patterns
        .iter()
        .map(|p| {
            // Resolve file path from file_id
            let file_path = p
                .file_id
                .and_then(|fid| {
                    file_repo
                        .get(codeilus_core::ids::FileId(fid))
                        .ok()
                        .map(|f| f.path)
                })
                .unwrap_or_default();
            PatternExport {
                kind: p.kind.clone(),
                severity: p.severity.clone(),
                message: p.description.clone(),
                file_path,
            }
        })
        .collect();

    // Load file metrics for hotspots
    let metrics_repo = codeilus_db::FileMetricsRepo::new(conn_arc.clone());
    let hotspot_rows = metrics_repo.list_hotspots(10)?;
    let all_metrics = metrics_repo.list()?;

    let avg_complexity = if all_metrics.is_empty() {
        0.0
    } else {
        all_metrics.iter().map(|m| m.complexity).sum::<f64>() / all_metrics.len() as f64
    };

    let hotspot_files: Vec<HotspotFile> = hotspot_rows
        .iter()
        .filter_map(|m| {
            file_repo.get(m.file_id).ok().map(|f| HotspotFile {
                path: f.path,
                heatmap_score: m.heatmap_score,
                complexity: m.complexity,
                churn: m.churn as usize,
            })
        })
        .collect();

    let total_sloc: usize = files.iter().map(|f| f.sloc as usize).sum();

    // Compute language badges
    let mut lang_counts: HashMap<String, usize> = HashMap::new();
    for f in &files {
        if let Some(ref lang) = f.language {
            *lang_counts.entry(lang.clone()).or_default() += 1;
        }
    }
    let total_with_lang = lang_counts.values().sum::<usize>().max(1);
    let mut language_badges: Vec<LanguageBadge> = lang_counts
        .iter()
        .map(|(lang, count)| LanguageBadge {
            language: lang.clone(),
            percentage: (*count as f64 / total_with_lang as f64) * 100.0,
            color: language_color(lang).to_string(),
        })
        .collect();
    language_badges.sort_by(|a, b| b.percentage.partial_cmp(&a.percentage).unwrap());

    // Build file tree from file paths
    let file_paths: Vec<String> = files.iter().map(|f| f.path.clone()).collect();
    let file_tree = build_simple_tree(&file_paths);

    // Architecture mermaid placeholder (codeilus-diagram requires KnowledgeGraph which we don't
    // have from DB alone — store empty string, the narrate crate may have generated one)
    let architecture_mermaid = String::new();

    let metrics_snapshot = MetricsSnapshot {
        total_files: files.len(),
        total_sloc,
        total_symbols,
        avg_complexity,
        modularity_q: 0.0,
        hotspot_files,
    };

    Ok(ExportData {
        repo_name: repo_name.to_string(),
        repo_description: None,
        language_badges,
        overview,
        architecture_mermaid,
        reading_order,
        entry_points: Vec::new(),
        architecture_narrative,
        extension_guide,
        contribution_guide,
        why_trending,
        metrics_snapshot,
        file_tree,
        communities,
        patterns,
    })
}

/// Build a simple ASCII file tree from sorted paths.
fn build_simple_tree(paths: &[String]) -> String {
    if paths.is_empty() {
        return String::new();
    }
    let mut sorted = paths.to_vec();
    sorted.sort();

    let mut lines = Vec::new();
    for (i, path) in sorted.iter().enumerate() {
        let is_last = i == sorted.len() - 1;
        let prefix = if is_last { "└── " } else { "├── " };
        lines.push(format!("{prefix}{path}"));
    }
    lines.join("\n")
}
