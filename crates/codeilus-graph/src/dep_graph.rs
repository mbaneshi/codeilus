use std::collections::HashMap;
use std::path::Path;

use codeilus_core::ids::FileId;
use codeilus_core::types::Language;
use codeilus_parse::ParsedFile;

/// A file dependency edge: (source_file_id, target_file_id).
pub type DepEdge = (FileId, FileId);

/// Build file-level dependency edges from import statements.
///
/// Matches import source strings to file paths in the parsed set,
/// using language-specific resolution strategies.
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
            if let Some(&target_id) =
                resolve_import(&import.from, &source_path, pf.language, file_index)
            {
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
/// Dispatches to language-specific resolvers for best results.
fn resolve_import<'a>(
    source: &str,
    source_file: &str,
    language: Language,
    file_index: &'a HashMap<String, FileId>,
) -> Option<&'a FileId> {
    // Direct path match (works for all languages)
    if let Some(id) = file_index.get(source) {
        return Some(id);
    }

    match language {
        Language::Rust => resolve_rust_import(source, source_file, file_index),
        Language::TypeScript | Language::JavaScript => {
            resolve_ts_import(source, source_file, file_index)
        }
        Language::Python => resolve_python_import(source, source_file, file_index),
        _ => resolve_generic_import(source, source_file, file_index),
    }
}

/// Resolve Rust `use` paths to filesystem paths.
///
/// Handles: `crate::foo::bar`, `self::module`, `super::module`,
/// and external crate references like `codeilus_core::types`.
fn resolve_rust_import<'a>(
    source: &str,
    source_file: &str,
    file_index: &'a HashMap<String, FileId>,
) -> Option<&'a FileId> {
    // Clean up use-tree braces: `crate::foo::{Bar, Baz}` → `crate::foo`
    let cleaned = if source.contains('{') {
        source
            .split("::{")
            .next()
            .unwrap_or(source)
            .trim_end_matches("::")
            .to_string()
    } else {
        source.to_string()
    };

    // Strip prefix and determine resolution strategy
    let (base_dir, module_path) = if let Some(rest) = cleaned.strip_prefix("crate::") {
        // Find the crate's src/ directory from the source file path
        let src_dir = find_crate_src_dir(source_file);
        (src_dir, rest)
    } else if let Some(rest) = cleaned.strip_prefix("self::") {
        // Relative to current module directory
        let parent = Path::new(source_file)
            .parent()
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        (Some(parent), rest)
    } else if let Some(rest) = cleaned.strip_prefix("super::") {
        // Up one directory
        let grandparent = Path::new(source_file)
            .parent()
            .and_then(|p| p.parent())
            .map(|p| p.to_string_lossy().to_string())
            .unwrap_or_default();
        (Some(grandparent), rest)
    } else {
        // External crate or unqualified — try matching as external crate
        // e.g., `codeilus_core::types` → look for `codeilus-core/src/types.rs`
        if let Some(result) = resolve_external_crate(&cleaned, file_index) {
            return Some(result);
        }
        // Fallback: try as module path segments
        (None, cleaned.as_str())
    };

    // Convert module path to filesystem candidates
    let segments: Vec<&str> = module_path.split("::").collect();

    // Try progressively shorter paths (last segments may be type names, not modules)
    for end in (1..=segments.len()).rev() {
        let fs_path = segments[..end].join("/");

        if let Some(ref base) = base_dir {
            // Try specific paths relative to base
            let candidates = [
                format!("{base}/{fs_path}.rs"),
                format!("{base}/{fs_path}/mod.rs"),
                format!("{base}/{fs_path}/lib.rs"),
            ];
            for candidate in &candidates {
                let normalized = candidate.replace("//", "/");
                if let Some(id) = file_index.get(&normalized) {
                    return Some(id);
                }
            }
        }

        // Try matching against all file paths (suffix match)
        for (file_path, id) in file_index {
            if file_path == source_file {
                continue;
            }
            let normalized = file_path.replace('\\', "/");

            let with_rs = format!("{fs_path}.rs");
            let with_mod = format!("{fs_path}/mod.rs");
            let with_lib = format!("{fs_path}/lib.rs");

            if normalized.ends_with(&format!("/{with_rs}"))
                || normalized.ends_with(&format!("/{with_mod}"))
                || normalized.ends_with(&format!("/{with_lib}"))
            {
                return Some(id);
            }

            // Also match just the last segment as filename (common for `use crate::types`)
            if end == 1 {
                let fname = normalized.rsplit('/').next().unwrap_or("");
                let stem = fname.strip_suffix(".rs").unwrap_or(fname);
                if stem == segments[0] {
                    return Some(id);
                }
            }
        }
    }

    None
}

/// Find the `src/` directory for the crate containing the given source file.
fn find_crate_src_dir(source_file: &str) -> Option<String> {
    let normalized = source_file.replace('\\', "/");
    // Look for `/src/` in the path — the crate root is everything up to and including `src`
    if let Some(pos) = normalized.rfind("/src/") {
        Some(normalized[..pos + 4].to_string()) // include `/src`
    } else if normalized.starts_with("src/") {
        Some("src".to_string())
    } else {
        None
    }
}

/// Resolve external Rust crate references.
/// e.g., `codeilus_core::types` → find `codeilus-core/src/types.rs`
fn resolve_external_crate<'a>(
    source: &str,
    file_index: &'a HashMap<String, FileId>,
) -> Option<&'a FileId> {
    let segments: Vec<&str> = source.split("::").collect();
    if segments.is_empty() {
        return None;
    }

    // The first segment is the crate name (with underscores)
    let crate_name_underscore = segments[0];
    let crate_name_hyphen = crate_name_underscore.replace('_', "-");

    // Remaining segments form the module path
    let module_segments = &segments[1..];

    if module_segments.is_empty() {
        // Just the crate name — try to find its lib.rs
        for (file_path, id) in file_index {
            let normalized = file_path.replace('\\', "/");
            if normalized.contains(&format!("/{crate_name_hyphen}/src/lib.rs"))
                || normalized.contains(&format!("/{crate_name_underscore}/src/lib.rs"))
            {
                return Some(id);
            }
        }
        return None;
    }

    // Try progressively shorter module paths
    for end in (1..=module_segments.len()).rev() {
        let fs_path = module_segments[..end].join("/");

        for (file_path, id) in file_index {
            let normalized = file_path.replace('\\', "/");

            // Match: crate-name/src/module/path.rs or .../mod.rs
            let patterns = [
                format!("/{crate_name_hyphen}/src/{fs_path}.rs"),
                format!("/{crate_name_hyphen}/src/{fs_path}/mod.rs"),
                format!("/{crate_name_underscore}/src/{fs_path}.rs"),
                format!("/{crate_name_underscore}/src/{fs_path}/mod.rs"),
            ];

            for pattern in &patterns {
                if normalized.ends_with(pattern) {
                    return Some(id);
                }
            }
        }
    }

    None
}

/// Resolve TypeScript/JavaScript imports.
///
/// Handles relative imports with extension resolution and index files.
fn resolve_ts_import<'a>(
    source: &str,
    source_file: &str,
    file_index: &'a HashMap<String, FileId>,
) -> Option<&'a FileId> {
    // Only resolve relative imports
    if !source.starts_with('.') {
        return None;
    }

    let source_dir = Path::new(source_file)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    // Normalize the relative path
    let resolved = resolve_relative_path(&source_dir, source);

    // Try with various extensions
    let extensions = [
        "", ".ts", ".tsx", ".js", ".jsx", ".mjs", ".cjs",
        "/index.ts", "/index.tsx", "/index.js", "/index.jsx",
    ];

    for ext in &extensions {
        let candidate = format!("{resolved}{ext}");
        let normalized = normalize_path(&candidate);
        if let Some(id) = file_index.get(&normalized) {
            return Some(id);
        }
    }

    None
}

/// Resolve Python imports.
///
/// Handles relative imports (`.module`, `..module`) and package imports.
fn resolve_python_import<'a>(
    source: &str,
    source_file: &str,
    file_index: &'a HashMap<String, FileId>,
) -> Option<&'a FileId> {
    let source_dir = Path::new(source_file)
        .parent()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default();

    if source.starts_with('.') {
        // Relative import: count leading dots
        let dots = source.chars().take_while(|c| *c == '.').count();
        let module = &source[dots..];

        let mut base = Path::new(&source_dir).to_path_buf();
        for _ in 1..dots {
            base = base.parent().unwrap_or(Path::new("")).to_path_buf();
        }
        let base_str = base.to_string_lossy().to_string();

        let module_path = module.replace('.', "/");
        let candidates = if module_path.is_empty() {
            vec![format!("{base_str}/__init__.py")]
        } else {
            vec![
                format!("{base_str}/{module_path}.py"),
                format!("{base_str}/{module_path}/__init__.py"),
            ]
        };

        for candidate in &candidates {
            let normalized = normalize_path(candidate);
            if let Some(id) = file_index.get(&normalized) {
                return Some(id);
            }
        }
    } else {
        // Absolute import: convert dots to path separators
        let module_path = source.replace('.', "/");
        let candidates = [
            format!("{module_path}.py"),
            format!("{module_path}/__init__.py"),
        ];

        for candidate in &candidates {
            if let Some(id) = file_index.get(candidate.as_str()) {
                return Some(id);
            }
            // Try with source dir prefix
            let with_base = format!("{source_dir}/{candidate}");
            let normalized = normalize_path(&with_base);
            if let Some(id) = file_index.get(&normalized) {
                return Some(id);
            }
        }

        // Suffix match against all file paths
        for (file_path, id) in file_index {
            let normalized = file_path.replace('\\', "/");
            if normalized.ends_with(&format!("/{module_path}.py"))
                || normalized.ends_with(&format!("/{module_path}/__init__.py"))
            {
                return Some(id);
            }
        }
    }

    None
}

/// Generic import resolution for other languages (Go, Java, etc.).
///
/// Tries common extension patterns and suffix matching.
fn resolve_generic_import<'a>(
    source: &str,
    source_file: &str,
    file_index: &'a HashMap<String, FileId>,
) -> Option<&'a FileId> {
    // Try common extensions
    for ext in &[".go", ".java", ".c", ".cpp", ".h", ".hpp", ".cs", ".rb", ".swift", ".kt"] {
        let with_ext = format!("{source}{ext}");
        if let Some(id) = file_index.get(&with_ext) {
            return Some(id);
        }
    }

    // Convert package-style paths to filesystem paths
    let fs_path = source.replace('.', "/");
    for ext in &[".go", ".java"] {
        for (file_path, id) in file_index {
            if file_path == source_file {
                continue;
            }
            let normalized = file_path.replace('\\', "/");
            if normalized.ends_with(&format!("/{fs_path}{ext}")) {
                return Some(id);
            }
        }
    }

    None
}

/// Resolve a relative path (`./foo` or `../bar`) against a base directory.
fn resolve_relative_path(base: &str, relative: &str) -> String {
    let mut parts: Vec<&str> = base.split('/').filter(|s| !s.is_empty()).collect();

    for segment in relative.split('/') {
        match segment {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }

    parts.join("/")
}

/// Normalize a path by resolving `.` and `..` and removing double slashes.
fn normalize_path(path: &str) -> String {
    let mut parts: Vec<&str> = Vec::new();
    for segment in path.split('/') {
        match segment {
            "." | "" => {}
            ".." => {
                parts.pop();
            }
            other => parts.push(other),
        }
    }
    parts.join("/")
}
