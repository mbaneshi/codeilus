//! Flowchart generation from source code via heuristic analysis.

use codeilus_core::CodeilusResult;
use codeilus_parse::Symbol;
use tracing::debug;

use crate::mermaid::{escape_label, sanitize_node_id};
use crate::types::{FlowEdge, FlowNode, FlowNodeKind, FlowchartIR};

/// Generate a Mermaid flowchart from a symbol and its source code.
pub fn generate(symbol: &Symbol, source: &str) -> CodeilusResult<FlowchartIR> {
    let ir = build_ir(symbol, source);
    debug!(
        symbol = %symbol.name,
        nodes = ir.nodes.len(),
        edges = ir.edges.len(),
        "built flowchart IR"
    );
    Ok(ir)
}

/// Convert a FlowchartIR to Mermaid syntax.
pub fn ir_to_mermaid(ir: &FlowchartIR) -> String {
    let mut output = String::from("flowchart TD\n");

    for node in &ir.nodes {
        let id = sanitize_node_id(&node.id);
        let label = escape_label(&node.label);
        let shape = match node.kind {
            FlowNodeKind::Entry => format!("    {}([\"{}\"]) ", id, label),
            FlowNodeKind::Exit => format!("    {}([\"{}\"])", id, label),
            FlowNodeKind::Process => format!("    {}[\"{}\"]\n", id, label),
            FlowNodeKind::Decision => format!("    {}{{\"{}\"}} ", id, label),
            FlowNodeKind::Loop => format!("    {}[[\"{}\"]]", id, label),
        };
        output.push_str(&shape);
        output.push('\n');
    }

    for edge in &ir.edges {
        let from = sanitize_node_id(&edge.from);
        let to = sanitize_node_id(&edge.to);
        match &edge.label {
            Some(label) => output.push_str(&format!("    {} -->|\"{}\"| {}\n", from, label, to)),
            None => output.push_str(&format!("    {} --> {}\n", from, to)),
        }
    }

    output
}

fn next_id(counter: &mut usize) -> String {
    let id = format!("node_{}", *counter);
    *counter += 1;
    id
}

fn flush_process(
    pending: &mut Vec<String>,
    nodes: &mut Vec<FlowNode>,
    edges: &mut Vec<FlowEdge>,
    prev: &mut String,
    counter: &mut usize,
) {
    if pending.is_empty() {
        return;
    }
    let id = next_id(counter);
    let label = pending.join("; ");
    nodes.push(FlowNode {
        id: id.clone(),
        kind: FlowNodeKind::Process,
        label,
    });
    edges.push(FlowEdge {
        from: prev.clone(),
        to: id.clone(),
        label: None,
    });
    *prev = id;
    pending.clear();
}

/// Build FlowchartIR from source code using heuristic line-by-line analysis.
fn build_ir(symbol: &Symbol, source: &str) -> FlowchartIR {
    let mut nodes = Vec::new();
    let mut edges = Vec::new();
    let mut counter: usize = 0;

    // Entry node
    let entry_id = next_id(&mut counter);
    nodes.push(FlowNode {
        id: entry_id.clone(),
        kind: FlowNodeKind::Entry,
        label: format!("Start: {}", symbol.name),
    });

    // Extract the relevant source lines
    let lines: Vec<&str> = source.lines().collect();
    let start = (symbol.start_line as usize).saturating_sub(1);
    let end = (symbol.end_line as usize).min(lines.len());
    let body_lines = if start < end { &lines[start..end] } else { &[] };

    let mut prev_id = entry_id;
    let mut pending_process_lines: Vec<String> = Vec::new();

    for line in body_lines {
        let trimmed = line.trim();

        // Skip empty lines, braces, comments, function signatures
        if trimmed.is_empty()
            || trimmed == "{"
            || trimmed == "}"
            || trimmed.starts_with("//")
            || trimmed.starts_with('#')
            || trimmed.starts_with("fn ")
            || trimmed.starts_with("def ")
            || trimmed.starts_with("func ")
            || trimmed.starts_with("function ")
            || trimmed.starts_with("pub fn ")
            || trimmed.starts_with("async fn ")
            || trimmed.starts_with("pub async fn ")
        {
            continue;
        }

        // Decision: if / else if / match / switch
        if trimmed.starts_with("if ")
            || trimmed.starts_with("} else if ")
            || trimmed.starts_with("else if ")
            || trimmed.starts_with("match ")
            || trimmed.starts_with("switch ")
            || trimmed.starts_with("switch(")
        {
            flush_process(
                &mut pending_process_lines,
                &mut nodes,
                &mut edges,
                &mut prev_id,
                &mut counter,
            );
            let id = next_id(&mut counter);
            let label = trimmed.trim_end_matches('{').trim().to_string();
            nodes.push(FlowNode {
                id: id.clone(),
                kind: FlowNodeKind::Decision,
                label,
            });
            edges.push(FlowEdge {
                from: prev_id.clone(),
                to: id.clone(),
                label: None,
            });
            prev_id = id;
            continue;
        }

        // Loop: for / while / loop
        if trimmed.starts_with("for ")
            || trimmed.starts_with("while ")
            || trimmed == "loop {"
            || trimmed == "loop"
        {
            flush_process(
                &mut pending_process_lines,
                &mut nodes,
                &mut edges,
                &mut prev_id,
                &mut counter,
            );
            let id = next_id(&mut counter);
            let label = trimmed.trim_end_matches('{').trim().to_string();
            nodes.push(FlowNode {
                id: id.clone(),
                kind: FlowNodeKind::Loop,
                label,
            });
            edges.push(FlowEdge {
                from: prev_id.clone(),
                to: id.clone(),
                label: None,
            });
            // Back-edge from loop to itself
            edges.push(FlowEdge {
                from: id.clone(),
                to: id.clone(),
                label: Some("repeat".to_string()),
            });
            prev_id = id;
            continue;
        }

        // Return statement → exit
        if trimmed.starts_with("return ") || trimmed == "return;" || trimmed == "return" {
            flush_process(
                &mut pending_process_lines,
                &mut nodes,
                &mut edges,
                &mut prev_id,
                &mut counter,
            );
            let id = next_id(&mut counter);
            nodes.push(FlowNode {
                id: id.clone(),
                kind: FlowNodeKind::Exit,
                label: trimmed.to_string(),
            });
            edges.push(FlowEdge {
                from: prev_id.clone(),
                to: id.clone(),
                label: None,
            });
            prev_id = id;
            continue;
        }

        // Everything else → accumulate as process
        pending_process_lines.push(trimmed.to_string());

        // Flush every 3 lines to avoid giant nodes
        if pending_process_lines.len() >= 3 {
            flush_process(
                &mut pending_process_lines,
                &mut nodes,
                &mut edges,
                &mut prev_id,
                &mut counter,
            );
        }
    }

    // Flush remaining process lines
    flush_process(
        &mut pending_process_lines,
        &mut nodes,
        &mut edges,
        &mut prev_id,
        &mut counter,
    );

    // Exit node
    let exit_id = next_id(&mut counter);
    nodes.push(FlowNode {
        id: exit_id.clone(),
        kind: FlowNodeKind::Exit,
        label: "End".to_string(),
    });
    edges.push(FlowEdge {
        from: prev_id,
        to: exit_id,
        label: None,
    });

    FlowchartIR { nodes, edges }
}

#[cfg(test)]
mod tests {
    use super::*;
    use codeilus_core::SymbolKind;

    fn make_symbol(name: &str, start: i64, end: i64) -> Symbol {
        Symbol {
            name: name.to_string(),
            kind: SymbolKind::Function,
            start_line: start,
            end_line: end,
            signature: None,
        }
    }

    #[test]
    fn flowchart_simple_function() {
        let source = "fn greet() {\n    let name = \"world\";\n    println!(\"Hello {}\", name);\n}\n";
        let symbol = make_symbol("greet", 1, 4);
        let ir = generate(&symbol, source).unwrap();

        assert!(ir.nodes.iter().any(|n| n.kind == FlowNodeKind::Entry));
        assert!(ir.nodes.iter().any(|n| n.kind == FlowNodeKind::Exit));
        assert!(ir.nodes.iter().any(|n| n.kind == FlowNodeKind::Process));
        assert!(!ir.nodes.iter().any(|n| n.kind == FlowNodeKind::Decision));
        assert!(!ir.nodes.iter().any(|n| n.kind == FlowNodeKind::Loop));

        let mermaid = ir_to_mermaid(&ir);
        assert!(mermaid.starts_with("flowchart TD"));
    }

    #[test]
    fn flowchart_if_else() {
        let source = r#"fn check(x: i32) {
    if x > 0 {
        println!("positive");
    } else {
        println!("non-positive");
    }
}
"#;
        let symbol = make_symbol("check", 1, 7);
        let ir = generate(&symbol, source).unwrap();

        assert!(
            ir.nodes.iter().any(|n| n.kind == FlowNodeKind::Decision),
            "Should have a decision node for if"
        );

        let mermaid = ir_to_mermaid(&ir);
        assert!(mermaid.contains('{'));
    }

    #[test]
    fn flowchart_for_loop() {
        let source = r#"fn sum(items: &[i32]) -> i32 {
    let mut total = 0;
    for item in items {
        total += item;
    }
    return total;
}
"#;
        let symbol = make_symbol("sum", 1, 7);
        let ir = generate(&symbol, source).unwrap();

        assert!(
            ir.nodes.iter().any(|n| n.kind == FlowNodeKind::Loop),
            "Should have a loop node"
        );

        let loop_node = ir.nodes.iter().find(|n| n.kind == FlowNodeKind::Loop).unwrap();
        let has_back_edge = ir
            .edges
            .iter()
            .any(|e| e.from == loop_node.id && e.to == loop_node.id);
        assert!(has_back_edge, "Loop should have a back-edge to itself");
    }

    #[test]
    fn flowchart_nested() {
        let source = r#"fn process(items: &[i32]) {
    for item in items {
        if *item > 0 {
            println!("positive: {}", item);
        }
    }
}
"#;
        let symbol = make_symbol("process", 1, 7);
        let ir = generate(&symbol, source).unwrap();

        assert!(ir.nodes.iter().any(|n| n.kind == FlowNodeKind::Loop));
        assert!(ir.nodes.iter().any(|n| n.kind == FlowNodeKind::Decision));

        let mermaid = ir_to_mermaid(&ir);
        assert!(mermaid.starts_with("flowchart TD"));
    }
}
