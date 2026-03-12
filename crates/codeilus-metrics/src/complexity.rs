use std::collections::HashMap;

use codeilus_core::ids::SymbolId;
use codeilus_core::Language;
use codeilus_parse::ParsedFile;

/// Branch keywords that increment cyclomatic complexity.
const BRANCH_KEYWORDS: &[&str] = &[
    "if", "else", "elif", "elsif", "match", "case", "for", "while", "try", "catch", "except",
    "&&", "||", "and", "or", "?",
];

/// Estimate cyclomatic complexity for each symbol.
///
/// If source text is available, counts branch keywords in the symbol's line range.
/// Otherwise, estimates from LOC: `1 + (loc / 10)`.
pub fn estimate_complexity(
    parsed_files: &[ParsedFile],
    sources: &HashMap<String, String>,
    symbol_ids: &HashMap<(String, String), SymbolId>,
) -> HashMap<SymbolId, f64> {
    let mut result = HashMap::new();

    for pf in parsed_files {
        let file_path = pf.path.to_string_lossy().to_string();
        let source = sources.get(&file_path);

        for sym in &pf.symbols {
            let key = (sym.name.clone(), file_path.clone());
            let symbol_id = match symbol_ids.get(&key) {
                Some(id) => *id,
                None => continue,
            };

            let loc = (sym.end_line - sym.start_line + 1).max(1) as usize;

            let complexity = if let Some(src) = source {
                estimate_from_source(src, sym.start_line, sym.end_line, pf.language)
            } else {
                1.0 + (loc as f64 / 10.0)
            };

            result.insert(symbol_id, complexity);
        }
    }

    result
}

/// Estimate complexity by counting branch keywords in source lines.
fn estimate_from_source(source: &str, start_line: i64, end_line: i64, _lang: Language) -> f64 {
    let lines: Vec<&str> = source.lines().collect();
    let start = (start_line - 1).max(0) as usize;
    let end = (end_line as usize).min(lines.len());

    let mut complexity = 1.0; // Base complexity

    for &line in &lines[start..end] {
        let trimmed = line.trim();
        for &keyword in BRANCH_KEYWORDS {
            if keyword.len() <= 2 {
                // Short operators: check for exact occurrence
                if trimmed.contains(keyword) {
                    complexity += trimmed.matches(keyword).count() as f64;
                }
            } else {
                // Keywords: check word boundary
                for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if word == keyword {
                        complexity += 1.0;
                    }
                }
            }
        }
    }

    complexity
}

/// Estimate complexity from LOC when source is unavailable.
pub fn estimate_from_loc(loc: usize) -> f64 {
    1.0 + (loc as f64 / 10.0)
}
