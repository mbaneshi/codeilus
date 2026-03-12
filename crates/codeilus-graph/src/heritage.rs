use std::collections::HashMap;

use codeilus_core::ids::SymbolId;
use codeilus_core::types::EdgeKind;
use codeilus_parse::ParsedFile;

/// A heritage edge: (child_id, parent_id, edge_kind).
pub type HeritageEdge = (SymbolId, SymbolId, EdgeKind);

/// Build heritage (extends/implements) edges from parsed files.
pub fn build_heritage_edges(
    parsed_files: &[ParsedFile],
    symbol_index: &HashMap<String, Vec<SymbolId>>,
) -> Vec<HeritageEdge> {
    let mut edges = Vec::new();

    for pf in parsed_files {
        for h in &pf.heritage {
            let child_ids = symbol_index.get(&h.child);
            let parent_ids = symbol_index.get(&h.parent);

            if let (Some(children), Some(parents)) = (child_ids, parent_ids) {
                // Take first match for each
                if let (Some(&child_id), Some(&parent_id)) =
                    (children.first(), parents.first())
                {
                    edges.push((child_id, parent_id, h.relation));
                }
            }
        }
    }

    edges
}
