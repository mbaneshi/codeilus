use codeilus_core::error::CodeilusResult;
use codeilus_core::SymbolKind;
use codeilus_parse::ParsedFile;

use crate::types::{PatternFinding, PatternKind, Severity};

/// Detect long methods — functions/methods with >50 lines of code.
pub fn detect(parsed_files: &[ParsedFile]) -> CodeilusResult<Vec<PatternFinding>> {
    let mut findings = Vec::new();

    for pf in parsed_files {
        let path = pf.path.to_string_lossy().to_string();

        for sym in &pf.symbols {
            if sym.kind != SymbolKind::Function && sym.kind != SymbolKind::Method {
                continue;
            }

            let lines = sym.end_line - sym.start_line;
            if lines <= 50 {
                continue;
            }

            let severity = if lines > 200 {
                Severity::Error
            } else if lines > 100 {
                Severity::Warning
            } else {
                Severity::Info
            };

            findings.push(PatternFinding {
                kind: PatternKind::LongMethod,
                severity,
                file_id: None,
                symbol_id: None,
                file_path: path.clone(),
                line: Some(sym.start_line as usize),
                message: format!("'{}' is {} lines long", sym.name, lines),
                suggestion: "Extract helper methods to improve readability".to_string(),
            });
        }
    }

    Ok(findings)
}
