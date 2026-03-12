use std::path::{Path, PathBuf};

use codeilus_core::Language;

/// Resolve an import source to a file path within the repository.
///
/// Returns `None` for external/third-party imports.
pub fn resolve_import(
    import_source: &str,
    file_path: &Path,
    repo_root: &Path,
    lang: Language,
) -> Option<String> {
    match lang {
        Language::Python => resolve_python(import_source, file_path, repo_root),
        Language::TypeScript | Language::JavaScript => {
            resolve_typescript(import_source, file_path, repo_root)
        }
        Language::Rust => resolve_rust(import_source, repo_root),
        Language::Go => resolve_go(import_source, repo_root),
        Language::Java => resolve_java(import_source, repo_root),
        _ => None,
    }
}

fn resolve_python(module: &str, file_path: &Path, repo_root: &Path) -> Option<String> {
    if module.starts_with('.') {
        // Relative import
        let dir = file_path.parent()?;
        let dots = module.chars().take_while(|c| *c == '.').count();
        let mut base = dir.to_path_buf();
        for _ in 1..dots {
            base = base.parent()?.to_path_buf();
        }
        let rest = &module[dots..];
        if rest.is_empty() {
            return try_python_path(&base, "__init__", repo_root);
        }
        let parts: Vec<&str> = rest.split('.').collect();
        let candidate = base.join(parts.join("/"));
        try_python_path(candidate.parent()?, candidate.file_name()?.to_str()?, repo_root)
    } else {
        // Absolute import — check if it's a local module
        let parts: Vec<&str> = module.split('.').collect();
        let candidate = repo_root.join(parts.join("/"));
        try_python_path(candidate.parent()?, candidate.file_name()?.to_str()?, repo_root)
    }
}

fn try_python_path(dir: &Path, name: &str, repo_root: &Path) -> Option<String> {
    let py_file = dir.join(format!("{name}.py"));
    if py_file.exists() {
        return make_relative(&py_file, repo_root);
    }
    let init_file = dir.join(name).join("__init__.py");
    if init_file.exists() {
        return make_relative(&init_file, repo_root);
    }
    None
}

fn resolve_typescript(source: &str, file_path: &Path, repo_root: &Path) -> Option<String> {
    if source.starts_with("./") || source.starts_with("../") {
        let dir = file_path.parent()?;
        let resolved = dir.join(source);
        for ext in &["", ".ts", ".tsx", ".js", ".jsx", "/index.ts", "/index.js"] {
            let candidate = PathBuf::from(format!("{}{}", resolved.display(), ext));
            if candidate.exists() {
                return make_relative(&candidate, repo_root);
            }
        }
    }
    // Bare imports are external
    None
}

fn resolve_rust(use_path: &str, repo_root: &Path) -> Option<String> {
    if let Some(rest) = use_path.strip_prefix("crate::") {
        let parts: Vec<&str> = rest.split("::").collect();
        if let Some(first) = parts.first() {
            let candidate = repo_root.join("src").join(format!("{first}.rs"));
            if candidate.exists() {
                return make_relative(&candidate, repo_root);
            }
            let mod_candidate = repo_root.join("src").join(first).join("mod.rs");
            if mod_candidate.exists() {
                return make_relative(&mod_candidate, repo_root);
            }
        }
    }
    None
}

fn resolve_go(import_path: &str, _repo_root: &Path) -> Option<String> {
    // Go imports are package paths — only local resolution is practical
    // External packages are not resolvable without go.mod
    if import_path.starts_with('.') {
        Some(import_path.to_string())
    } else {
        None
    }
}

fn resolve_java(import_path: &str, repo_root: &Path) -> Option<String> {
    // Convert package.Class to path
    let parts: Vec<&str> = import_path.split('.').collect();
    let candidate = repo_root
        .join("src/main/java")
        .join(parts.join("/"))
        .with_extension("java");
    if candidate.exists() {
        return make_relative(&candidate, repo_root);
    }
    // Try without src/main/java
    let candidate = repo_root.join(parts.join("/")).with_extension("java");
    if candidate.exists() {
        return make_relative(&candidate, repo_root);
    }
    None
}

fn make_relative(path: &Path, root: &Path) -> Option<String> {
    path.strip_prefix(root)
        .ok()
        .map(|p| p.to_string_lossy().to_string())
}
