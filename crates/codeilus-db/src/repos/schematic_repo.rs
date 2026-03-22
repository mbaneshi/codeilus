//! Schematic repository: depth-limited tree with community + learning enrichment.

use codeilus_core::error::{CodeilusError, CodeilusResult};
use rusqlite::params;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Arc;

use crate::pool::DbPool;

#[derive(Debug, Clone, Serialize)]
pub struct SchematicNode {
    pub id: String,
    #[serde(rename = "type")]
    pub node_type: String,
    pub label: String,
    pub parent_id: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sloc: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signature: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub community_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub community_label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub community_color: Option<String>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressInfo>,

    pub has_children: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub child_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_count: Option<usize>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ProgressInfo {
    pub completed: i64,
    pub total: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchematicEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    #[serde(rename = "type")]
    pub edge_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<f64>,
}

#[derive(Debug, Clone, Serialize)]
pub struct CommunityInfo {
    pub id: i64,
    pub label: String,
    pub color: String,
    pub cohesion: f64,
    pub member_count: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter_title: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub difficulty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub progress: Option<ProgressInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchematicMeta {
    pub total_files: i64,
    pub total_symbols: i64,
    pub total_communities: i64,
    pub depth_returned: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchematicResponse {
    pub nodes: Vec<SchematicNode>,
    pub edges: Vec<SchematicEdge>,
    pub communities: Vec<CommunityInfo>,
    pub meta: SchematicMeta,
}

#[derive(Debug, Clone, Serialize)]
pub struct SchematicDetail {
    pub node_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrative: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub narrative_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<SourceInfo>,
    pub callers: Vec<RelatedSymbol>,
    pub callees: Vec<RelatedSymbol>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chapter: Option<ChapterInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceInfo {
    pub path: String,
    pub language: Option<String>,
    pub lines: Vec<SourceLine>,
    pub total_lines: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SourceLine {
    pub number: i64,
    pub content: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct RelatedSymbol {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub file_path: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ChapterInfo {
    pub id: i64,
    pub title: String,
    pub difficulty: String,
    pub progress: ProgressInfo,
}

const COMMUNITY_COLORS: &[&str] = &[
    "#6366f1", "#ec4899", "#14b8a6", "#f59e0b", "#8b5cf6",
    "#06b6d4", "#f97316", "#84cc16", "#ef4444", "#a855f7",
];

fn community_color(id: i64) -> String {
    COMMUNITY_COLORS[(id as usize) % COMMUNITY_COLORS.len()].to_string()
}

pub struct SchematicRepo {
    db: Arc<DbPool>,
}

impl SchematicRepo {
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// Build the schematic tree up to `depth` levels.
    pub fn get_schematic(
        &self,
        depth: u32,
        community_filter: Option<i64>,
        include_symbols: bool,
        include_edges: bool,
    ) -> CodeilusResult<SchematicResponse> {
        let conn = self.db.connection();

        // 1. Load files (with optional community filter)
        let files: Vec<(i64, String, Option<String>, i64)> = if let Some(cid) = community_filter {
            let mut stmt = conn.prepare(
                "SELECT DISTINCT f.id, f.path, f.language, f.sloc
                 FROM files f
                 JOIN symbols s ON s.file_id = f.id
                 JOIN community_members cm ON cm.symbol_id = s.id
                 WHERE cm.community_id = ?1
                 ORDER BY f.path"
            ).map_err(db_err)?;
            let rows = stmt.query_map(params![cid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
                .map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
            rows
        } else {
            let mut stmt = conn.prepare(
                "SELECT id, path, language, sloc FROM files ORDER BY path"
            ).map_err(db_err)?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
                .map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
            rows
        };

        // 2. Load symbol → community mapping
        let sym_community: HashMap<i64, i64> = {
            let mut stmt = conn.prepare("SELECT symbol_id, community_id FROM community_members").map_err(db_err)?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
                .map_err(db_err)?.collect::<Result<HashMap<_, _>, _>>().map_err(db_err)?;
            rows
        };

        // 3. Load community info
        let communities_raw: Vec<(i64, Option<String>, f64)> = {
            let mut stmt = conn.prepare("SELECT id, name, cohesion_score FROM communities").map_err(db_err)?;
            let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get::<_, f64>(2).unwrap_or(0.0))))
                .map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
            rows
        };

        // 4. Community member counts
        let community_member_counts: HashMap<i64, i64> = {
            let mut stmt = conn.prepare("SELECT community_id, COUNT(*) FROM community_members GROUP BY community_id").map_err(db_err)?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, i64>(1)?)))
                .map_err(db_err)?.collect::<Result<HashMap<_, _>, _>>().map_err(db_err)?;
            rows
        };

        // 5. Chapters linked to communities
        let community_chapters: HashMap<i64, (i64, String, String)> = {
            let mut stmt = conn.prepare("SELECT community_id, id, title, difficulty FROM chapters WHERE community_id IS NOT NULL").map_err(db_err)?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, (r.get(1)?, r.get(2)?, r.get(3)?))))
                .map_err(db_err)?.collect::<Result<HashMap<_, _>, _>>().map_err(db_err)?;
            rows
        };

        // 6. Chapter progress
        let chapter_progress: HashMap<i64, (i64, i64)> = {
            let mut stmt = conn.prepare(
                "SELECT cs.chapter_id,
                        COUNT(cs.id) as total,
                        SUM(CASE WHEN p.completed = 1 THEN 1 ELSE 0 END) as done
                 FROM chapter_sections cs
                 LEFT JOIN progress p ON p.chapter_id = cs.chapter_id AND p.section_id = cs.id
                 GROUP BY cs.chapter_id"
            ).map_err(db_err)?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, (r.get::<_, i64>(1)?, r.get::<_, i64>(2)?))))
                .map_err(db_err)?.collect::<Result<HashMap<_, _>, _>>().map_err(db_err)?;
            rows
        };

        // 7. File → dominant community (most symbols in same community)
        let file_dominant_community: HashMap<i64, i64> = {
            let mut stmt = conn.prepare(
                "SELECT s.file_id, cm.community_id, COUNT(*) as cnt
                 FROM symbols s
                 JOIN community_members cm ON cm.symbol_id = s.id
                 GROUP BY s.file_id, cm.community_id
                 ORDER BY s.file_id, cnt DESC"
            ).map_err(db_err)?;
            let rows: Vec<(i64, i64, i64)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?)))
                .map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
            let mut result = HashMap::new();
            for (file_id, comm_id, _cnt) in rows {
                result.entry(file_id).or_insert(comm_id);
            }
            result
        };

        // 8. Symbol counts per file
        let file_symbol_counts: HashMap<i64, usize> = {
            let mut stmt = conn.prepare("SELECT file_id, COUNT(*) FROM symbols GROUP BY file_id").map_err(db_err)?;
            let rows = stmt.query_map([], |r| Ok((r.get::<_, i64>(0)?, r.get::<_, usize>(1)?)))
                .map_err(db_err)?.collect::<Result<HashMap<_, _>, _>>().map_err(db_err)?;
            rows
        };

        // Build community info list
        let community_map: HashMap<i64, String> = communities_raw.iter()
            .map(|(id, name, _)| (*id, name.clone().unwrap_or_else(|| format!("community_{}", id))))
            .collect();

        let communities: Vec<CommunityInfo> = communities_raw.iter().map(|(id, name, cohesion)| {
            let label = name.clone().unwrap_or_else(|| format!("community_{}", id));
            let (chap_id, chap_title, diff) = community_chapters.get(id)
                .map(|(cid, title, d)| (Some(*cid), Some(title.clone()), Some(d.clone())))
                .unwrap_or((None, None, None));
            let progress = chap_id.and_then(|cid| chapter_progress.get(&cid))
                .map(|(total, completed)| ProgressInfo { completed: *completed, total: *total });
            CommunityInfo {
                id: *id, label, color: community_color(*id), cohesion: *cohesion,
                member_count: *community_member_counts.get(id).unwrap_or(&0),
                chapter_id: chap_id, chapter_title: chap_title, difficulty: diff, progress,
            }
        }).collect();

        // Build directory tree nodes
        let mut nodes: Vec<SchematicNode> = Vec::new();
        let mut dir_children: HashMap<String, Vec<String>> = HashMap::new(); // dir_id → child dir_ids

        // Root node
        nodes.push(SchematicNode {
            id: "dir:.".into(), node_type: "directory".into(), label: ".".into(),
            parent_id: None, has_children: true, child_count: None, symbol_count: None,
            file_id: None, symbol_id: None, language: None, sloc: None,
            kind: None, signature: None,
            community_id: None, community_label: None, community_color: None,
            chapter_id: None, chapter_title: None, difficulty: None, progress: None,
        });

        // Find common prefix for path normalization
        let common_prefix = if files.len() > 1 {
            let paths: Vec<&str> = files.iter().map(|(_, p, _, _)| p.as_str()).collect();
            find_common_dir_prefix(&paths)
        } else {
            String::new()
        };

        // Process files into directory tree
        let mut all_dirs: HashMap<String, bool> = HashMap::new();
        all_dirs.insert(".".to_string(), true);

        for (fid, path, lang, sloc) in &files {
            let clean = path.strip_prefix(&common_prefix).unwrap_or(path);
            let clean = clean.strip_prefix('/').unwrap_or(clean);
            let clean = clean.strip_prefix("./").unwrap_or(clean);
            let parts: Vec<&str> = clean.split('/').collect();

            // Create intermediate directory nodes
            let mut current_dir = ".".to_string();
            for part in &parts[..parts.len() - 1] {
                let dir_path = if current_dir == "." {
                    (*part).to_string()
                } else {
                    format!("{}/{}", current_dir, part)
                };
                let dir_id = format!("dir:{}", dir_path);
                let parent_id = format!("dir:{}", current_dir);

                let depth_of_dir = dir_path.matches('/').count() as u32 + 1;
                if depth_of_dir <= depth && !all_dirs.contains_key(&dir_path) {
                    all_dirs.insert(dir_path.clone(), true);
                    nodes.push(SchematicNode {
                        id: dir_id.clone(), node_type: "directory".into(),
                        label: (*part).to_string(), parent_id: Some(parent_id.clone()),
                        has_children: true, child_count: None, symbol_count: None,
                        file_id: None, symbol_id: None, language: None, sloc: None,
                        kind: None, signature: None,
                        community_id: None, community_label: None, community_color: None,
                        chapter_id: None, chapter_title: None, difficulty: None, progress: None,
                    });
                    dir_children.entry(parent_id).or_default().push(dir_id);
                }
                current_dir = dir_path;
            }

            // Add file node if within depth
            let file_depth = parts.len() as u32 - 1;
            if file_depth <= depth {
                let file_id_str = format!("file:{}", fid);
                let parent_id = format!("dir:{}", current_dir);
                let comm_id = file_dominant_community.get(fid);
                let comm_label = comm_id.and_then(|c| community_map.get(c)).cloned();
                let comm_color = comm_id.map(|c| community_color(*c));

                nodes.push(SchematicNode {
                    id: file_id_str, node_type: "file".into(),
                    label: parts.last().unwrap_or(&"").to_string(),
                    parent_id: Some(parent_id),
                    has_children: file_symbol_counts.get(fid).copied().unwrap_or(0) > 0,
                    child_count: None,
                    symbol_count: file_symbol_counts.get(fid).copied(),
                    file_id: Some(*fid), symbol_id: None,
                    language: lang.clone(), sloc: Some(*sloc),
                    kind: None, signature: None,
                    community_id: comm_id.copied(),
                    community_label: comm_label,
                    community_color: comm_color,
                    chapter_id: None, chapter_title: None, difficulty: None, progress: None,
                });
            }
        }

        // Optionally load symbols (filtered by community if set)
        if include_symbols {
            #[allow(clippy::type_complexity)]
            let syms: Vec<(i64, i64, String, String, Option<i32>, Option<i32>, Option<String>)> = if let Some(cid) = community_filter {
                let mut stmt = conn.prepare(
                    "SELECT s.id, s.file_id, s.name, s.kind, s.start_line, s.end_line, s.signature
                     FROM symbols s
                     JOIN community_members cm ON cm.symbol_id = s.id
                     WHERE cm.community_id = ?1
                     ORDER BY s.file_id, s.start_line"
                ).map_err(db_err)?;
                let rows = stmt.query_map(params![cid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)))
                    .map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
                rows
            } else {
                let mut stmt = conn.prepare(
                    "SELECT id, file_id, name, kind, start_line, end_line, signature FROM symbols ORDER BY file_id, start_line"
                ).map_err(db_err)?;
                let rows = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?, r.get(6)?)))
                    .map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
                rows
            };
            for (sid, fid, name, kind, _start, _end, sig) in syms {
                let comm_id = sym_community.get(&sid);
                let comm_label = comm_id.and_then(|c| community_map.get(c)).cloned();
                let comm_color = comm_id.map(|c| community_color(*c));
                nodes.push(SchematicNode {
                    id: format!("sym:{}", sid), node_type: "symbol".into(),
                    label: name, parent_id: Some(format!("file:{}", fid)),
                    has_children: false, child_count: None, symbol_count: None,
                    file_id: Some(fid), symbol_id: Some(sid),
                    language: None, sloc: None,
                    kind: Some(kind), signature: sig,
                    community_id: comm_id.copied(),
                    community_label: comm_label,
                    community_color: comm_color,
                    chapter_id: None, chapter_title: None, difficulty: None, progress: None,
                });
            }

        }

        // Enrich directory nodes with dominant community using ALL files
        {
            let mut dir_community_counts: HashMap<String, HashMap<i64, usize>> = HashMap::new();
            // Use all files (not just returned nodes) to compute dir communities
            for (fid, path, _lang, _sloc) in &files {
                if let Some(&comm_id) = file_dominant_community.get(fid) {
                    let clean = path.strip_prefix("./").unwrap_or(path);
                    let parts: Vec<&str> = clean.split('/').collect();
                    // Attribute to every ancestor directory
                    let mut dir_path = ".".to_string();
                    for part in parts.iter().take(parts.len() - 1) {
                        dir_path = if dir_path == "." { (*part).to_string() } else { format!("{}/{}", dir_path, part) };
                        let dir_id = format!("dir:{}", dir_path);
                        *dir_community_counts.entry(dir_id).or_default().entry(comm_id).or_default() += 1;
                    }
                }
            }
            for node in &mut nodes {
                if node.node_type == "directory" && node.community_id.is_none() {
                    if let Some(counts) = dir_community_counts.get(&node.id) {
                        if let Some((&best_id, _)) = counts.iter().max_by_key(|(_, c)| *c) {
                            node.community_id = Some(best_id);
                            node.community_label = community_map.get(&best_id).cloned();
                            node.community_color = Some(community_color(best_id));
                        }
                    }
                }
            }
        }

        // Optionally load edges (filtered by community if set)
        let edges = if include_edges {
            if let Some(cid) = community_filter {
                // Only edges where both endpoints are in this community
                let mut stmt = conn.prepare(
                    "SELECT e.id, e.source_id, e.target_id, e.kind, e.confidence
                     FROM edges e
                     JOIN community_members cm1 ON cm1.symbol_id = e.source_id AND cm1.community_id = ?1
                     JOIN community_members cm2 ON cm2.symbol_id = e.target_id AND cm2.community_id = ?1"
                ).map_err(db_err)?;
                let rows = stmt.query_map(params![cid], |r| {
                    Ok(SchematicEdge {
                        id: format!("e:{}", r.get::<_, i64>(0)?),
                        source: format!("sym:{}", r.get::<_, i64>(1)?),
                        target: format!("sym:{}", r.get::<_, i64>(2)?),
                        edge_type: r.get(3)?,
                        confidence: r.get(4).ok(),
                    })
                }).map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
                rows
            } else {
                let mut stmt = conn.prepare("SELECT id, source_id, target_id, kind, confidence FROM edges").map_err(db_err)?;
                let rows = stmt.query_map([], |r| {
                    Ok(SchematicEdge {
                        id: format!("e:{}", r.get::<_, i64>(0)?),
                        source: format!("sym:{}", r.get::<_, i64>(1)?),
                        target: format!("sym:{}", r.get::<_, i64>(2)?),
                        edge_type: r.get(3)?,
                        confidence: r.get(4).ok(),
                    })
                }).map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
                rows
            }
        } else {
            Vec::new()
        };

        // Counts
        let total_files: i64 = conn.query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0)).unwrap_or(0);
        let total_symbols: i64 = conn.query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0)).unwrap_or(0);
        let total_communities: i64 = conn.query_row("SELECT COUNT(*) FROM communities", [], |r| r.get(0)).unwrap_or(0);

        Ok(SchematicResponse {
            nodes,
            edges,
            communities,
            meta: SchematicMeta { total_files, total_symbols, total_communities, depth_returned: depth },
        })
    }

    /// Expand children of a specific node.
    pub fn expand_node(
        &self,
        node_id: &str,
        include_symbols: bool,
        include_edges: bool,
    ) -> CodeilusResult<SchematicResponse> {
        let conn = self.db.connection();

        let mut nodes = Vec::new();
        let mut edges = Vec::new();

        if let Some(dir_path) = node_id.strip_prefix("dir:") {
            // Load ALL files, find common prefix, then filter to this directory's children
            let mut stmt = conn.prepare("SELECT id, path, language, sloc FROM files ORDER BY path")
                .map_err(db_err)?;
            let all_files: Vec<(i64, String, Option<String>, i64)> = stmt.query_map([], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?)))
                .map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;

            // Find common prefix to normalize paths (handles both "./crates/..." and "/abs/path/..." )
            let common_prefix = if all_files.len() > 1 {
                let paths: Vec<&str> = all_files.iter().map(|(_, p, _, _)| p.as_str()).collect();
                find_common_dir_prefix(&paths)
            } else {
                String::new()
            };

            // Extract direct children: subdirs and files
            let mut seen_dirs: HashMap<String, bool> = HashMap::new();
            for (fid, path, lang, sloc) in &all_files {
                let clean = path.strip_prefix(&common_prefix).unwrap_or(path);
                let clean = clean.strip_prefix('/').unwrap_or(clean);
                let clean = clean.strip_prefix("./").unwrap_or(clean);

                let relative = if dir_path == "." {
                    clean.to_string()
                } else {
                    match clean.strip_prefix(&format!("{}/", dir_path)) {
                        Some(r) => r.to_string(),
                        None => continue,
                    }
                };

                let parts: Vec<&str> = relative.split('/').collect();
                if parts.len() == 1 && !parts[0].is_empty() {
                    nodes.push(SchematicNode {
                        id: format!("file:{}", fid), node_type: "file".into(),
                        label: parts[0].to_string(), parent_id: Some(node_id.to_string()),
                        has_children: false, child_count: None, symbol_count: None,
                        file_id: Some(*fid), symbol_id: None,
                        language: lang.clone(), sloc: Some(*sloc),
                        kind: None, signature: None,
                        community_id: None, community_label: None, community_color: None,
                        chapter_id: None, chapter_title: None, difficulty: None, progress: None,
                    });
                } else if parts.len() > 1 {
                    let sub_dir = parts[0];
                    let sub_dir_path = if dir_path == "." { sub_dir.to_string() } else { format!("{}/{}", dir_path, sub_dir) };
                    if !seen_dirs.contains_key(&sub_dir_path) {
                        seen_dirs.insert(sub_dir_path.clone(), true);
                        nodes.push(SchematicNode {
                            id: format!("dir:{}", sub_dir_path), node_type: "directory".into(),
                            label: sub_dir.to_string(), parent_id: Some(node_id.to_string()),
                            has_children: true, child_count: None, symbol_count: None,
                            file_id: None, symbol_id: None, language: None, sloc: None,
                            kind: None, signature: None,
                            community_id: None, community_label: None, community_color: None,
                            chapter_id: None, chapter_title: None, difficulty: None, progress: None,
                        });
                    }
                }
            }
        } else if let Some(file_id_str) = node_id.strip_prefix("file:") {
            let file_id: i64 = file_id_str.parse().map_err(|_| CodeilusError::Validation("invalid file id".into()))?;
            if include_symbols {
                let mut stmt = conn.prepare(
                    "SELECT id, name, kind, start_line, end_line, signature FROM symbols WHERE file_id = ?1 ORDER BY start_line"
                ).map_err(db_err)?;
                #[allow(clippy::type_complexity)]
                let syms: Vec<(i64, String, String, Option<i32>, Option<i32>, Option<String>)> = stmt
                    .query_map(params![file_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?, r.get(5)?)))
                    .map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;

                let sym_ids: Vec<i64> = syms.iter().map(|(id, ..)| *id).collect();

                for (sid, name, kind, _start, _end, sig) in syms {
                    nodes.push(SchematicNode {
                        id: format!("sym:{}", sid), node_type: "symbol".into(),
                        label: name, parent_id: Some(node_id.to_string()),
                        has_children: false, child_count: None, symbol_count: None,
                        file_id: Some(file_id), symbol_id: Some(sid),
                        language: None, sloc: None,
                        kind: Some(kind), signature: sig,
                        community_id: None, community_label: None, community_color: None,
                        chapter_id: None, chapter_title: None, difficulty: None, progress: None,
                    });
                }

                if include_edges && !sym_ids.is_empty() {
                    let placeholders: String = sym_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
                    let sql = format!(
                        "SELECT id, source_id, target_id, kind, confidence FROM edges WHERE source_id IN ({0}) OR target_id IN ({0})",
                        placeholders
                    );
                    let mut stmt = conn.prepare(&sql).map_err(db_err)?;
                    let params: Vec<Box<dyn rusqlite::types::ToSql>> = sym_ids.iter().map(|id| Box::new(*id) as Box<dyn rusqlite::types::ToSql>).collect();
                    let double_params: Vec<&dyn rusqlite::types::ToSql> = params.iter().chain(params.iter()).map(|b| b.as_ref()).collect();
                    edges = stmt.query_map(double_params.as_slice(), |r| {
                        Ok(SchematicEdge {
                            id: format!("e:{}", r.get::<_, i64>(0)?),
                            source: format!("sym:{}", r.get::<_, i64>(1)?),
                            target: format!("sym:{}", r.get::<_, i64>(2)?),
                            edge_type: r.get(3)?,
                            confidence: r.get(4).ok(),
                        })
                    }).map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
                }
            }
        }

        Ok(SchematicResponse {
            nodes,
            edges,
            communities: Vec::new(),
            meta: SchematicMeta { total_files: 0, total_symbols: 0, total_communities: 0, depth_returned: 0 },
        })
    }

    /// Get detailed info for a node (narrative, source, callers/callees, chapter).
    pub fn get_detail(&self, node_id: &str, include_source: bool) -> CodeilusResult<SchematicDetail> {
        let conn = self.db.connection();

        if let Some(sym_id_str) = node_id.strip_prefix("sym:") {
            let sym_id: i64 = sym_id_str.parse().map_err(|_| CodeilusError::Validation("invalid symbol id".into()))?;

            // Get symbol info
            let (file_id, _name, _kind, _start_line, _end_line): (i64, String, String, Option<i64>, Option<i64>) = conn.query_row(
                "SELECT file_id, name, kind, start_line, end_line FROM symbols WHERE id = ?1",
                params![sym_id], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?, r.get(3)?, r.get(4)?))
            ).map_err(db_err)?;

            // Narrative
            let narrative: Option<(String, String)> = conn.query_row(
                "SELECT content, kind FROM narratives WHERE target_id = ?1 AND kind = 'symbol_explanation'",
                params![sym_id], |r| Ok((r.get(0)?, r.get(1)?))
            ).ok();

            // Source
            let source = if include_source {
                let (path, lang): (String, Option<String>) = conn.query_row(
                    "SELECT path, language FROM files WHERE id = ?1", params![file_id], |r| Ok((r.get(0)?, r.get(1)?))
                ).map_err(db_err)?;
                // Read source file if repo_root available — for now return empty
                Some(SourceInfo { path, language: lang, lines: Vec::new(), total_lines: 0 })
            } else { None };

            // Callers
            let callers = self.get_related_symbols(&conn, sym_id, true)?;
            let callees = self.get_related_symbols(&conn, sym_id, false)?;

            // Chapter (via community)
            let chapter = self.get_symbol_chapter(&conn, sym_id)?;

            Ok(SchematicDetail {
                node_id: node_id.to_string(),
                narrative: narrative.as_ref().map(|(c, _)| c.clone()),
                narrative_kind: narrative.map(|(_, k)| k),
                source, callers, callees, chapter,
            })
        } else if let Some(file_id_str) = node_id.strip_prefix("file:") {
            let file_id: i64 = file_id_str.parse().map_err(|_| CodeilusError::Validation("invalid file id".into()))?;

            let narrative: Option<(String, String)> = conn.query_row(
                "SELECT content, kind FROM narratives WHERE target_id = ?1 AND kind = 'file_overview'",
                params![file_id], |r| Ok((r.get(0)?, r.get(1)?))
            ).ok();

            Ok(SchematicDetail {
                node_id: node_id.to_string(),
                narrative: narrative.as_ref().map(|(c, _)| c.clone()),
                narrative_kind: narrative.map(|(_, k)| k),
                source: None, callers: Vec::new(), callees: Vec::new(), chapter: None,
            })
        } else {
            Ok(SchematicDetail {
                node_id: node_id.to_string(),
                narrative: None, narrative_kind: None, source: None,
                callers: Vec::new(), callees: Vec::new(), chapter: None,
            })
        }
    }

    fn get_related_symbols(&self, conn: &rusqlite::Connection, sym_id: i64, callers: bool) -> CodeilusResult<Vec<RelatedSymbol>> {
        let sql = if callers {
            "SELECT s.id, s.name, s.kind, f.path FROM edges e JOIN symbols s ON s.id = e.source_id JOIN files f ON f.id = s.file_id WHERE e.target_id = ?1 LIMIT 20"
        } else {
            "SELECT s.id, s.name, s.kind, f.path FROM edges e JOIN symbols s ON s.id = e.target_id JOIN files f ON f.id = s.file_id WHERE e.source_id = ?1 LIMIT 20"
        };
        let mut stmt = conn.prepare(sql).map_err(db_err)?;
        let rows = stmt.query_map(params![sym_id], |r| Ok(RelatedSymbol {
            id: format!("sym:{}", r.get::<_, i64>(0)?),
            name: r.get(1)?, kind: r.get(2)?, file_path: r.get(3)?,
        })).map_err(db_err)?.collect::<Result<Vec<_>, _>>().map_err(db_err)?;
        Ok(rows)
    }

    fn get_symbol_chapter(&self, conn: &rusqlite::Connection, sym_id: i64) -> CodeilusResult<Option<ChapterInfo>> {
        let comm_id: Option<i64> = conn.query_row(
            "SELECT community_id FROM community_members WHERE symbol_id = ?1 LIMIT 1",
            params![sym_id], |r| r.get(0)
        ).ok();

        if let Some(cid) = comm_id {
            let chapter: Option<(i64, String, String)> = conn.query_row(
                "SELECT id, title, difficulty FROM chapters WHERE community_id = ?1",
                params![cid], |r| Ok((r.get(0)?, r.get(1)?, r.get(2)?))
            ).ok();

            if let Some((ch_id, title, diff)) = chapter {
                let (total, completed): (i64, i64) = conn.query_row(
                    "SELECT COUNT(cs.id), SUM(CASE WHEN p.completed = 1 THEN 1 ELSE 0 END)
                     FROM chapter_sections cs
                     LEFT JOIN progress p ON p.chapter_id = cs.chapter_id AND p.section_id = cs.id
                     WHERE cs.chapter_id = ?1",
                    params![ch_id], |r| Ok((r.get(0)?, r.get::<_, i64>(1).unwrap_or(0)))
                ).unwrap_or((0, 0));

                return Ok(Some(ChapterInfo { id: ch_id, title, difficulty: diff, progress: ProgressInfo { completed, total } }));
            }
        }
        Ok(None)
    }
}

fn db_err(e: rusqlite::Error) -> CodeilusError {
    CodeilusError::Database(Box::new(e))
}

/// Find the longest common directory prefix among a set of file paths.
fn find_common_dir_prefix(paths: &[&str]) -> String {
    if paths.is_empty() { return String::new(); }
    let first = paths[0];
    let mut prefix_len = 0;
    for (i, ch) in first.char_indices() {
        if paths.iter().all(|p| p.as_bytes().get(i) == Some(&(ch as u8))) {
            if ch == '/' { prefix_len = i + 1; }
        } else {
            break;
        }
    }
    first[..prefix_len].to_string()
}
