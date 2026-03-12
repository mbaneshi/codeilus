use codeilus_core::Language;
use std::collections::HashMap;

/// Count non-blank, non-comment lines for a given source and language.
pub fn count_sloc(source: &str, lang: Language) -> usize {
    let mut in_block_comment = false;
    let mut count = 0;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        match lang {
            Language::Python => {
                if trimmed.starts_with('#') {
                    continue;
                }
                // Triple-quote block comments (simplified)
                if trimmed.starts_with("\"\"\"") || trimmed.starts_with("'''") {
                    let quote = &trimmed[..3];
                    if in_block_comment {
                        in_block_comment = false;
                        continue;
                    }
                    // Check if it closes on same line
                    if trimmed.len() > 3 && trimmed[3..].contains(quote) {
                        continue; // Single-line docstring
                    }
                    in_block_comment = true;
                    continue;
                }
                if in_block_comment {
                    continue;
                }
                count += 1;
            }
            Language::Rust
            | Language::TypeScript
            | Language::JavaScript
            | Language::Go
            | Language::Java
            | Language::C
            | Language::Cpp
            | Language::CSharp
            | Language::Swift
            | Language::Kotlin => {
                if in_block_comment {
                    if trimmed.contains("*/") {
                        in_block_comment = false;
                    }
                    continue;
                }
                if trimmed.starts_with("//") {
                    continue;
                }
                if trimmed.starts_with("/*") {
                    if !trimmed.contains("*/") {
                        in_block_comment = true;
                    }
                    continue;
                }
                count += 1;
            }
            Language::Ruby => {
                if in_block_comment {
                    if trimmed == "=end" {
                        in_block_comment = false;
                    }
                    continue;
                }
                if trimmed == "=begin" {
                    in_block_comment = true;
                    continue;
                }
                if trimmed.starts_with('#') {
                    continue;
                }
                count += 1;
            }
            Language::PHP => {
                if in_block_comment {
                    if trimmed.contains("*/") {
                        in_block_comment = false;
                    }
                    continue;
                }
                if trimmed.starts_with("//") || trimmed.starts_with('#') {
                    continue;
                }
                if trimmed.starts_with("/*") {
                    if !trimmed.contains("*/") {
                        in_block_comment = true;
                    }
                    continue;
                }
                count += 1;
            }
        }
    }

    count
}

/// Aggregate language breakdown from parsed file data.
pub fn language_breakdown(files: &[(String, Language, usize)]) -> HashMap<String, usize> {
    let mut breakdown = HashMap::new();
    for (_, lang, sloc) in files {
        *breakdown.entry(lang.as_str().to_string()).or_default() += sloc;
    }
    breakdown
}
