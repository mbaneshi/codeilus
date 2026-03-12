use codeilus_core::error::CodeilusResult;
use codeilus_core::SymbolKind;
use codeilus_parse::ParsedFile;

use crate::types::{PatternFinding, PatternKind, Severity};

/// Detect god classes — classes/structs with >20 methods.
pub fn detect(parsed_files: &[ParsedFile]) -> CodeilusResult<Vec<PatternFinding>> {
    let mut findings = Vec::new();

    for file in parsed_files {
        let path = file.path.to_string_lossy().to_string();

        // Find all class/struct symbols
        let classes: Vec<_> = file
            .symbols
            .iter()
            .filter(|s| s.kind == SymbolKind::Class || s.kind == SymbolKind::Struct)
            .collect();

        for class in &classes {
            // Count methods within this class's line range
            let method_count = file
                .symbols
                .iter()
                .filter(|s| {
                    s.kind == SymbolKind::Method
                        && s.start_line >= class.start_line
                        && s.end_line <= class.end_line
                })
                .count();

            if method_count > 20 {
                let severity = if method_count > 30 {
                    Severity::Error
                } else {
                    Severity::Warning
                };

                findings.push(PatternFinding {
                    kind: PatternKind::GodClass,
                    severity,
                    file_id: None,
                    symbol_id: None,
                    file_path: path.clone(),
                    line: Some(class.start_line as usize),
                    message: format!(
                        "'{}' has {} methods — consider splitting",
                        class.name, method_count
                    ),
                    suggestion: "Consider splitting into smaller classes with single responsibilities".to_string(),
                });
            }
        }
    }

    Ok(findings)
}
