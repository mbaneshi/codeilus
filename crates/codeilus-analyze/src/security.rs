use codeilus_core::error::CodeilusResult;
use codeilus_parse::ParsedFile;
use regex::Regex;

use crate::types::{PatternFinding, PatternKind, Severity};

struct SecurityPattern {
    regex: Regex,
    severity: Severity,
    message: &'static str,
    suggestion: &'static str,
}

/// Detect security hotspots via regex scanning of file paths.
/// Since ParsedFile doesn't carry source content, we scan symbol names/signatures
/// and use path-based heuristics. For full content scanning, accept (path, content) pairs.
pub fn detect(parsed_files: &[ParsedFile]) -> CodeilusResult<Vec<PatternFinding>> {
    // No source content available in ParsedFile, so return empty.
    // The content-based scanner is available via detect_in_content().
    let _ = parsed_files;
    Ok(Vec::new())
}

/// Scan source file contents for security hotspots.
pub fn detect_in_content(
    files: &[(String, String)],
) -> CodeilusResult<Vec<PatternFinding>> {
    let patterns = build_patterns();
    let mut findings = Vec::new();

    for (path, content) in files {
        for (line_num, line) in content.lines().enumerate() {
            for pat in &patterns {
                if pat.regex.is_match(line) {
                    findings.push(PatternFinding {
                        kind: PatternKind::SecurityHotspot,
                        severity: pat.severity,
                        file_id: None,
                        symbol_id: None,
                        file_path: path.clone(),
                        line: Some(line_num + 1),
                        message: pat.message.to_string(),
                        suggestion: pat.suggestion.to_string(),
                    });
                }
            }
        }
    }

    Ok(findings)
}

fn build_patterns() -> Vec<SecurityPattern> {
    vec![
        SecurityPattern {
            regex: Regex::new(r"\beval\s*\(").unwrap(),
            severity: Severity::Warning,
            message: "Dynamic code execution via eval()",
            suggestion: "Avoid eval() — use safer alternatives like JSON.parse or AST-based evaluation",
        },
        SecurityPattern {
            regex: Regex::new(r"\bexec\s*\(").unwrap(),
            severity: Severity::Warning,
            message: "Dynamic code execution via exec()",
            suggestion: "Avoid exec() — use safer alternatives",
        },
        SecurityPattern {
            regex: Regex::new(r#"(?i)(password|api_key|secret)\s*=\s*["']"#).unwrap(),
            severity: Severity::Error,
            message: "Hardcoded secret",
            suggestion: "Use environment variables or a secrets manager instead of hardcoded values",
        },
        SecurityPattern {
            regex: Regex::new(r#"f["']SELECT\b|f["']INSERT\b|f["']UPDATE\b|f["']DELETE\b|\bSELECT\b.*\+\s*\w|\bSELECT\b.*\{.*\}"#).unwrap(),
            severity: Severity::Warning,
            message: "Potential SQL injection",
            suggestion: "Use parameterized queries instead of string concatenation",
        },
        SecurityPattern {
            regex: Regex::new(r"\b(subprocess|os\.system|child_process)\b").unwrap(),
            severity: Severity::Warning,
            message: "Command injection risk",
            suggestion: "Sanitize inputs and avoid shell=True; use subprocess with a list of arguments",
        },
        SecurityPattern {
            regex: Regex::new(r"\b(innerHTML|dangerouslySetInnerHTML)\b").unwrap(),
            severity: Severity::Warning,
            message: "XSS risk",
            suggestion: "Sanitize HTML content before rendering; prefer textContent or safe rendering APIs",
        },
    ]
}
