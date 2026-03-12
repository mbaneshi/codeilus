use rusqlite::{params, Transaction};

use codeilus_core::{CodeilusError, Confidence, EdgeId, EdgeKind, SymbolId};

pub struct NewEdge {
    pub source_id: SymbolId,
    pub target_id: SymbolId,
    pub kind: EdgeKind,
    pub confidence: Confidence,
}

pub struct EdgeRepo;

impl EdgeRepo {
    pub fn insert_edges(
        &self,
        tx: &Transaction,
        edges: &[NewEdge],
    ) -> Result<Vec<EdgeId>, CodeilusError> {
        let mut stmt = tx
            .prepare(
                "INSERT INTO edges (source_id, target_id, kind, confidence) VALUES (?1, ?2, ?3, ?4)",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(edges.len());

        for edge in edges {
            stmt.execute(params![
                edge.source_id.0,
                edge.target_id.0,
                format!("{:?}", edge.kind),
                edge.confidence.0,
            ])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            let id = tx.last_insert_rowid();
            ids.push(EdgeId(id));
        }

        Ok(ids)
    }
}

