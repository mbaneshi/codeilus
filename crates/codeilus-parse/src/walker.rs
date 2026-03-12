use std::path::{Path, PathBuf};

use codeilus_core::CodeilusError;
use ignore::WalkBuilder;

use crate::language;

pub fn walk_files(
    root: &Path,
    follow_symlinks: bool,
    max_file_bytes: usize,
) -> Result<Vec<PathBuf>, CodeilusError> {
    let mut builder = WalkBuilder::new(root);
    builder
        .follow_links(follow_symlinks)
        .git_ignore(true)
        .git_exclude(true)
        .hidden(false);

    let mut out = Vec::new();
    for result in builder.build() {
        let entry = match result {
            Ok(e) => e,
            Err(err) => {
                tracing::warn!("walker error: {err}");
                continue;
            }
        };
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        if language::detect_language(path).is_none() {
            continue;
        }
        if let Ok(md) = std::fs::metadata(path) {
            if md.len() as usize > max_file_bytes {
                continue;
            }
        }
        out.push(path.to_path_buf());
    }
    Ok(out)
}

