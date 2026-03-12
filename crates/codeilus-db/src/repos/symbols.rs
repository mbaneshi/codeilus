use rusqlite::{params, Transaction};

use codeilus_core::{CodeilusError, FileId, SymbolId, SymbolKind};

pub struct NewSymbol {
    pub file_id: FileId,
    pub name: String,
    pub kind: SymbolKind,
    pub start_line: i64,
    pub end_line: i64,
    pub signature: Option<String>,
}

pub struct SymbolRepo;

impl SymbolRepo {
    pub fn insert_symbols(
        &self,
        tx: &Transaction,
        symbols: &[NewSymbol],
    ) -> Result<Vec<SymbolId>, CodeilusError> {
        let mut stmt = tx
            .prepare(
                "INSERT INTO symbols (file_id, name, kind, start_line, end_line, signature) \
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(symbols.len());
        for sym in symbols {
            stmt.execute(params![
                sym.file_id.0,
                sym.name,
                format!("{:?}", sym.kind),
                sym.start_line,
                sym.end_line,
                sym.signature
            ])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            let id = tx.last_insert_rowid();
            ids.push(SymbolId(id));
        }
        Ok(ids)
    }
}

