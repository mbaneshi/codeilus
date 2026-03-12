use codeilus_core::types::Language;
use std::collections::HashMap;

/// Count source lines of code, excluding blank lines and comments.
pub fn count_sloc(source: &str, lang: Language) -> usize {
    let mut count = 0;
    let mut in_block_comment = false;

    for line in source.lines() {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            continue;
        }

        match lang {
            Language::Python | Language::Ruby => {
                if !trimmed.starts_with('#') {
                    count += 1;
                }
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
            | Language::Kotlin
            | Language::PHP => {
                if in_block_comment {
                    if trimmed.contains("*/") {
                        in_block_comment = false;
                    }
                    continue;
                }
                if trimmed.starts_with("/*") {
                    if !trimmed.contains("*/") {
                        in_block_comment = true;
                    }
                    continue;
                }
                if trimmed.starts_with("//") {
                    continue;
                }
                count += 1;
            }
        }
    }

    count
}

/// Compute language breakdown: language name → total SLOC.
pub fn language_breakdown(
    files: &[(Language, usize)],
) -> HashMap<String, usize> {
    let mut breakdown = HashMap::new();
    for (lang, sloc) in files {
        *breakdown.entry(lang.as_str().to_string()).or_default() += sloc;
    }
    breakdown
}
