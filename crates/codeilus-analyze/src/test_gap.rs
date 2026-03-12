use codeilus_core::error::CodeilusResult;
use codeilus_parse::ParsedFile;
use std::collections::HashSet;
use std::path::Path;

use crate::types::{PatternFinding, PatternKind, Severity};

/// Detect files that likely need tests but don't have them.
pub fn detect(parsed_files: &[ParsedFile]) -> CodeilusResult<Vec<PatternFinding>> {
    let mut findings = Vec::new();

    // Collect all known file paths for fast lookup
    let all_paths: HashSet<String> = parsed_files
        .iter()
        .map(|f| f.path.to_string_lossy().to_string())
        .collect();

    for file in parsed_files {
        // Skip files that are themselves test files
        let path_str = file.path.to_string_lossy().to_string();
        if is_test_file(&path_str) {
            continue;
        }

        // Only flag files with >5 symbols
        if file.symbols.len() <= 5 {
            continue;
        }

        // Check if a corresponding test file exists
        let test_candidates = generate_test_paths(&file.path);
        let has_test = test_candidates.iter().any(|t| all_paths.contains(t));

        if !has_test {
            findings.push(PatternFinding {
                kind: PatternKind::TestGap,
                severity: Severity::Info,
                file_id: None,
                symbol_id: None,
                file_path: path_str,
                line: None,
                message: format!(
                    "No test file found — this file has {} public symbols",
                    file.symbols.len()
                ),
                suggestion: format!(
                    "Add tests for this file — it has {} public symbols",
                    file.symbols.len()
                ),
            });
        }
    }

    Ok(findings)
}

fn is_test_file(path: &str) -> bool {
    let lower = path.to_lowercase();
    lower.contains("test_")
        || lower.contains("_test.")
        || lower.contains(".test.")
        || lower.contains(".spec.")
        || lower.contains("/tests/")
        || lower.contains("/test/")
        || lower.starts_with("test_")
}

fn generate_test_paths(path: &Path) -> Vec<String> {
    let mut candidates = Vec::new();
    let path_str = path.to_string_lossy().to_string();

    let stem = path.file_stem().unwrap_or_default().to_string_lossy();
    let ext = path.extension().unwrap_or_default().to_string_lossy();
    let parent = path.parent().unwrap_or(Path::new(""));

    // Python: test_foo.py, foo_test.py, tests/foo.py, tests/test_foo.py
    if ext == "py" {
        candidates.push(parent.join(format!("test_{stem}.py")).to_string_lossy().to_string());
        candidates.push(parent.join(format!("{stem}_test.py")).to_string_lossy().to_string());
        candidates.push(parent.join(format!("tests/{stem}.py")).to_string_lossy().to_string());
        candidates.push(parent.join(format!("tests/test_{stem}.py")).to_string_lossy().to_string());
        // Also from project root
        candidates.push(format!("tests/test_{stem}.py"));
        candidates.push(format!("tests/{stem}.py"));
    }

    // TypeScript/JavaScript: foo.test.ts, foo.spec.ts
    if ext == "ts" || ext == "tsx" || ext == "js" || ext == "jsx" {
        candidates.push(parent.join(format!("{stem}.test.{ext}")).to_string_lossy().to_string());
        candidates.push(parent.join(format!("{stem}.spec.{ext}")).to_string_lossy().to_string());
    }

    // Go: foo_test.go
    if ext == "go" {
        candidates.push(parent.join(format!("{stem}_test.go")).to_string_lossy().to_string());
    }

    // Rust: uses mod tests inside the file, but also check for tests/ dir
    if ext == "rs" {
        candidates.push(format!("tests/{stem}.rs"));
        // Check if there's a tests directory parallel to src
        if path_str.contains("src/") {
            let test_path = path_str.replace("src/", "tests/");
            candidates.push(test_path);
        }
    }

    candidates
}
