use std::collections::HashMap;

use codeilus_core::ids::SymbolId;

/// Branch keywords that increase cyclomatic complexity.
const BRANCH_KEYWORDS: &[&str] = &[
    "if", "else", "elif", "match", "case", "for", "while", "try", "catch",
    "except", "&&", "||", "?",
];

/// Estimate cyclomatic complexity for a symbol given its source lines.
pub fn estimate_complexity(source_lines: &[&str]) -> f64 {
    let mut complexity = 1.0;

    for line in source_lines {
        let trimmed = line.trim();
        for kw in BRANCH_KEYWORDS {
            // Check for keyword as a whole word or operator
            if kw.len() <= 2 {
                // Operators: &&, ||, ?
                if trimmed.contains(kw) {
                    complexity += 1.0;
                }
            } else {
                // Keywords: check word boundaries
                for word in trimmed.split(|c: char| !c.is_alphanumeric() && c != '_') {
                    if word == *kw {
                        complexity += 1.0;
                    }
                }
            }
        }
    }

    complexity
}

/// Fallback: estimate complexity from lines of code.
pub fn estimate_from_loc(loc: usize) -> f64 {
    1.0 + (loc as f64 / 10.0)
}

/// Compute complexity for all symbols.
///
/// `symbol_sources` maps SymbolId to the source lines of that symbol.
/// If source is not available, falls back to LOC-based estimate.
pub fn compute_complexity(
    symbol_sources: &HashMap<SymbolId, Vec<String>>,
    symbol_locs: &HashMap<SymbolId, usize>,
) -> HashMap<SymbolId, f64> {
    let mut result = HashMap::new();

    for (id, loc) in symbol_locs {
        if let Some(lines) = symbol_sources.get(id) {
            let line_refs: Vec<&str> = lines.iter().map(|s| s.as_str()).collect();
            result.insert(*id, estimate_complexity(&line_refs));
        } else {
            result.insert(*id, estimate_from_loc(*loc));
        }
    }

    result
}
