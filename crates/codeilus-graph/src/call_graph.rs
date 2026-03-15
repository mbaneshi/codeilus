use std::collections::HashMap;

use codeilus_core::ids::SymbolId;
use codeilus_core::types::Confidence;
use codeilus_parse::ParsedFile;

/// A resolved call edge: (caller_id, callee_id, confidence).
pub type CallEdge = (SymbolId, SymbolId, Confidence);

/// Build call graph edges from parsed files.
///
/// Maps caller/callee names to SymbolIds using the symbol index.
/// Confidence: exact match (same file) = 1.0, name-only = 0.7, ambiguous = 0.4.
pub fn build_call_edges(
    parsed_files: &[ParsedFile],
    symbol_index: &HashMap<String, Vec<SymbolId>>,
    name_to_id: &HashMap<(String, String), SymbolId>,
) -> Vec<CallEdge> {
    let mut edges = Vec::new();

    for pf in parsed_files {
        let file_path = pf.path.to_string_lossy().to_string();

        for call in &pf.calls {
            // Resolve caller — handle <module> by using first symbol in same file
            let caller_id = if call.caller == "<module>" {
                name_to_id
                    .iter()
                    .find(|((_, path), _)| path == &file_path)
                    .map(|(_, &id)| (id, Confidence(0.5)))
            } else {
                resolve_symbol(
                    &call.caller,
                    &file_path,
                    symbol_index,
                    name_to_id,
                )
            };

            // Resolve callee
            let callee_id = resolve_symbol(
                &call.callee,
                &file_path,
                symbol_index,
                name_to_id,
            );

            if let (Some((caller, _)), Some((callee, confidence))) = (caller_id, callee_id) {
                edges.push((caller, callee, confidence));
            }
        }
    }

    edges
}

/// Resolve a symbol name to a SymbolId with confidence scoring.
fn resolve_symbol(
    name: &str,
    file_path: &str,
    symbol_index: &HashMap<String, Vec<SymbolId>>,
    name_to_id: &HashMap<(String, String), SymbolId>,
) -> Option<(SymbolId, Confidence)> {
    // Try exact match: same name + same file
    let key = (name.to_string(), file_path.to_string());
    if let Some(&id) = name_to_id.get(&key) {
        return Some((id, Confidence::certain()));
    }

    // Try name-only match
    if let Some(ids) = symbol_index.get(name) {
        if ids.len() == 1 {
            return Some((ids[0], Confidence(0.7)));
        }
        if !ids.is_empty() {
            // Ambiguous: multiple symbols with same name
            return Some((ids[0], Confidence(0.4)));
        }
    }

    None
}
