use std::path::Path;

use codeilus_core::Language;

pub fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Language::from_extension(&ext)
}

pub fn is_supported_extension(ext: &str) -> bool {
    matches!(
        ext.to_ascii_lowercase().as_str(),
        "py" | "ts" | "tsx" | "js" | "jsx" | "rs" | "go" | "java"
    )
}

