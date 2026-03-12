//! Mermaid validation and escaping utilities.

use crate::types::ValidationResult;

/// Validate Mermaid diagram syntax.
pub fn validate(mermaid: &str) -> ValidationResult {
    let mut errors = Vec::new();
    let lines: Vec<&str> = mermaid.lines().collect();

    if lines.is_empty() {
        errors.push("Empty diagram".to_string());
        return ValidationResult {
            valid: false,
            errors,
        };
    }

    // Check for valid graph/flowchart/subgraph keywords on first non-empty line
    let first_line = lines
        .iter()
        .find(|l| !l.trim().is_empty())
        .map(|l| l.trim())
        .unwrap_or("");
    let valid_starts = [
        "graph ", "graph\t", "flowchart ", "flowchart\t", "sequenceDiagram", "classDiagram",
        "stateDiagram", "erDiagram", "gantt", "pie",
    ];
    if !valid_starts.iter().any(|s| first_line.starts_with(s)) {
        errors.push(format!(
            "Diagram must start with a valid keyword (graph, flowchart, etc.), found: '{}'",
            first_line.chars().take(40).collect::<String>()
        ));
    }

    // Check balanced brackets
    check_balanced(mermaid, '(', ')', "parentheses", &mut errors);
    check_balanced(mermaid, '[', ']', "square brackets", &mut errors);
    check_balanced(mermaid, '{', '}', "curly braces", &mut errors);

    // Check balanced quotes
    let quote_count = mermaid.chars().filter(|&c| c == '"').count();
    if quote_count % 2 != 0 {
        errors.push("Unbalanced double quotes".to_string());
    }

    // Check edges use valid syntax
    for (i, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        if trimmed.contains("->") && !trimmed.contains("-->") && !trimmed.contains("-.->") {
            // Single arrow -> is not valid Mermaid edge syntax (except inside labels)
            // Only flag if it looks like an edge (has content on both sides)
            if trimmed.contains(" -> ") {
                errors.push(format!(
                    "Line {}: Invalid edge syntax '->'. Use '-->', '-.->',  or '==>'",
                    i + 1
                ));
            }
        }
    }

    // Check subgraph/end balance
    let subgraph_count = lines
        .iter()
        .filter(|l| l.trim().starts_with("subgraph "))
        .count();
    let end_count = lines
        .iter()
        .filter(|l| l.trim() == "end")
        .count();
    if subgraph_count != end_count {
        errors.push(format!(
            "Unbalanced subgraph/end: {} subgraph(s) but {} end(s)",
            subgraph_count, end_count
        ));
    }

    ValidationResult {
        valid: errors.is_empty(),
        errors,
    }
}

fn check_balanced(input: &str, open: char, close: char, name: &str, errors: &mut Vec<String>) {
    // Skip characters inside double-quoted strings
    let mut depth: i32 = 0;
    let mut in_quotes = false;
    for ch in input.chars() {
        if ch == '"' {
            in_quotes = !in_quotes;
        } else if !in_quotes {
            if ch == open {
                depth += 1;
            } else if ch == close {
                depth -= 1;
                if depth < 0 {
                    errors.push(format!("Unbalanced {}: extra closing '{}'", name, close));
                    return;
                }
            }
        }
    }
    if depth > 0 {
        errors.push(format!("Unbalanced {}: missing closing '{}'", name, close));
    }
}

/// Escape special characters in a Mermaid label.
pub fn escape_label(label: &str) -> String {
    let mut result = String::with_capacity(label.len());
    let truncated = if label.len() > 60 {
        &label[..57]
    } else {
        label
    };

    for ch in truncated.chars() {
        match ch {
            '"' => result.push_str("#quot;"),
            '(' => result.push_str("#lpar;"),
            ')' => result.push_str("#rpar;"),
            '[' => result.push_str("#lsqb;"),
            ']' => result.push_str("#rsqb;"),
            '{' => result.push_str("#lbrace;"),
            '}' => result.push_str("#rbrace;"),
            '<' => result.push_str("#lt;"),
            '>' => result.push_str("#gt;"),
            '\n' => result.push_str("<br/>"),
            '\r' => {}
            _ => result.push(ch),
        }
    }

    if label.len() > 60 {
        result.push_str("...");
    }

    result
}

/// Sanitize a string for use as a Mermaid node ID.
pub fn sanitize_node_id(id: &str) -> String {
    let mut result = String::with_capacity(id.len());
    for ch in id.chars() {
        if ch.is_ascii_alphanumeric() || ch == '_' {
            result.push(ch);
        } else {
            result.push('_');
        }
    }
    // Ensure starts with a letter
    if result
        .chars()
        .next()
        .is_none_or(|c| !c.is_ascii_alphabetic())
    {
        result.insert(0, 'n');
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mermaid_valid() {
        let input = r#"graph TD
    n1["parse_file (fn)"]
    n2["Parser (struct)"]
    n1 --> n2"#;
        let result = validate(input);
        assert!(result.valid, "errors: {:?}", result.errors);
        assert!(result.errors.is_empty());
    }

    #[test]
    fn mermaid_unbalanced_brackets() {
        let input = r#"graph TD
    n1["parse_file (fn)"]
    n2["Parser (struct)"
    n1 --> n2"#;
        let result = validate(input);
        assert!(!result.valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn mermaid_escape_label() {
        let escaped = escape_label(r#"handle("request")"#);
        assert!(!escaped.contains('"'));
        assert!(!escaped.contains('('));
        assert!(!escaped.contains(')'));
        assert!(escaped.contains("#quot;"));
        assert!(escaped.contains("#lpar;"));
        assert!(escaped.contains("#rpar;"));
    }

    #[test]
    fn mermaid_sanitize_id() {
        assert_eq!(sanitize_node_id("hello world"), "hello_world");
        assert_eq!(sanitize_node_id("123abc"), "n123abc");
        assert_eq!(sanitize_node_id("foo::bar"), "foo__bar");
        assert_eq!(sanitize_node_id("valid_id"), "valid_id");
    }

    #[test]
    fn escape_long_label() {
        let long = "a".repeat(100);
        let escaped = escape_label(&long);
        assert!(escaped.ends_with("..."));
        assert!(escaped.len() < 70);
    }

    #[test]
    fn escape_newlines() {
        let escaped = escape_label("line1\nline2");
        assert!(escaped.contains("<br/>"));
    }
}
