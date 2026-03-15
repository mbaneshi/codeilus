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
            if let Some(&target_id) = resolve_import(&import.from, &source_path, file_index) {
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
///
/// Handles multiple resolution strategies:
/// 1. Direct path match
/// 2. Rust module paths (crate::, super::, self::, qualified paths)
/// 3. Common extensions
/// 4. Filename stem matching
fn resolve_import<'a>(
    source: &str,
    source_file: &str,
    file_index: &'a HashMap<String, FileId>,
) -> Option<&'a FileId> {
    // Direct path match
    if let Some(id) = file_index.get(source) {
        return Some(id);
    }

    // Rust module path resolution: crate::foo::bar → find files containing "foo/bar"
    let cleaned = source
        .replace("::{", "")
        .replace(['{', '}'], "")
        .replace(", ", "::");

    // Strip common Rust prefixes
    let module_path = if let Some(rest) = cleaned.strip_prefix("crate::") {
        rest
    } else if let Some(rest) = cleaned.strip_prefix("self::") {
        rest
    } else if let Some(rest) = cleaned.strip_prefix("super::") {
        rest
    } else {
        &cleaned
    };

    // Convert Rust path segments to filesystem-like patterns
    // e.g., "graph::builder::GraphBuilder" → try matching "graph/builder" or "graph.rs"
    let segments: Vec<&str> = module_path.split("::").collect();

    if !segments.is_empty() {
        // Try progressively shorter path suffixes
        // For "graph::builder::GraphBuilder", try:
        //   "graph/builder" (module path)
        //   "graph" (parent module)
        for end in (1..=segments.len()).rev() {
            let path_pattern = segments[..end].join("/");

            for (file_path, id) in file_index {
                // Skip matching against self
                if file_path == source_file {
                    continue;
                }

                // Check if file path contains the module path pattern
                // e.g., "crates/codeilus-graph/src/builder.rs" contains "builder"
                // e.g., "crates/codeilus-graph/src/graph/mod.rs" contains "graph"
                let normalized = file_path.replace('\\', "/");

                // Match: path ends with "segments/file.rs" or "segments.rs" or "segments/mod.rs"
                let with_rs = format!("{path_pattern}.rs");
                let with_mod = format!("{path_pattern}/mod.rs");
                let with_lib = format!("{path_pattern}/lib.rs");

                if normalized.ends_with(&with_rs)
                    || normalized.ends_with(&with_mod)
                    || normalized.ends_with(&with_lib)
                {
                    return Some(id);
                }

                // Also try matching just the last segment as filename
                if end == 1 {
                    let last_seg = segments[0];
                    let fname = normalized.rsplit('/').next().unwrap_or("");
                    let stem = fname.strip_suffix(".rs").unwrap_or(fname);
                    if stem == last_seg {
                        return Some(id);
                    }
                }
            }
        }
    }

    // Try common extensions (for non-Rust languages)
    for ext in &[".py", ".ts", ".tsx", ".js", ".jsx", ".go", ".java"] {
        let with_ext = format!("{source}{ext}");
        if let Some(id) = file_index.get(&with_ext) {
            return Some(id);
        }
    }

    // Try matching just the filename portion (fallback)
    let source_stem = source
        .rsplit("::")
        .next()
        .unwrap_or(source)
        .rsplit('/')
        .next()
        .unwrap_or(source);
    // Skip single-character or very generic stems
    if source_stem.len() >= 3 {
        for (path, id) in file_index {
            if path == source_file {
                continue;
            }
            let path_stem = path.rsplit('/').next().unwrap_or(path);
            let path_name = path_stem.strip_suffix(".rs")
                .or_else(|| path_stem.strip_suffix(".py"))
                .or_else(|| path_stem.strip_suffix(".ts"))
                .or_else(|| path_stem.strip_suffix(".tsx"))
                .or_else(|| path_stem.strip_suffix(".js"))
                .or_else(|| path_stem.strip_suffix(".go"))
                .or_else(|| path_stem.strip_suffix(".java"))
                .unwrap_or(path_stem);
            if path_name == source_stem.to_lowercase() {
                return Some(id);
            }
        }
    }

    None
}
