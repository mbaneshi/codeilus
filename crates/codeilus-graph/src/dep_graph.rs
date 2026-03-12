use std::collections::HashMap;

use codeilus_core::ids::FileId;
use codeilus_parse::ParsedFile;

/// A file dependency edge: (source_file_id, target_file_id).
pub type DepEdge = (FileId, FileId);

/// Build file-level dependency edges from import statements.
///
/// Matches import source strings to file paths in the parsed set.
pub fn build_dep_edges(
    parsed_files: &[ParsedFile],
    file_index: &HashMap<String, FileId>,
) -> Vec<DepEdge> {
    let mut edges = Vec::new();

    for pf in parsed_files {
        let source_path = pf.path.to_string_lossy().to_string();
        let source_id = match file_index.get(&source_path) {
            Some(id) => *id,
            None => continue,
        };

        for import in &pf.imports {
            // Try to match import source to a known file
            if let Some(&target_id) = resolve_import(&import.from, file_index) {
                if source_id != target_id {
                    edges.push((source_id, target_id));
                }
            }
        }
    }

    // Dedup
    edges.sort_by_key(|e| (e.0 .0, e.1 .0));
    edges.dedup();
    edges
}

/// Try to match an import source string to a file in the index.
fn resolve_import<'a>(
    source: &str,
    file_index: &'a HashMap<String, FileId>,
) -> Option<&'a FileId> {
    // Direct path match
    if let Some(id) = file_index.get(source) {
        return Some(id);
    }

    // Try common extensions
    for ext in &[".py", ".ts", ".tsx", ".js", ".jsx", ".rs", ".go", ".java"] {
        let with_ext = format!("{source}{ext}");
        if let Some(id) = file_index.get(&with_ext) {
            return Some(id);
        }
    }

    // Try matching just the filename portion
    let source_stem = source.rsplit('/').next().unwrap_or(source);
    for (path, id) in file_index {
        let path_stem = path.rsplit('/').next().unwrap_or(path);
        let path_name = path_stem.rsplit('.').next_back().unwrap_or(path_stem);
        if path_name == source_stem {
            return Some(id);
        }
    }

    None
}
