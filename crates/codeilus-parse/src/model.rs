use std::path::PathBuf;

use codeilus_core::{Confidence, EdgeKind, Language, SymbolKind};

#[derive(Debug, Clone)]
pub struct Symbol {
    pub name: String,
    pub kind: SymbolKind,
    pub start_line: i64,
    pub end_line: i64,
    pub signature: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Import {
    pub from: String,
    pub name: String,
}

#[derive(Debug, Clone)]
pub struct Call {
    pub caller: String,
    pub callee: String,
    pub line: i64,
}

#[derive(Debug, Clone)]
pub struct Heritage {
    pub child: String,
    pub parent: String,
    pub relation: EdgeKind,
    pub confidence: Confidence,
}

#[derive(Debug, Clone)]
pub struct ParsedFile {
    pub path: PathBuf,
    pub language: Language,
    pub symbols: Vec<Symbol>,
    pub imports: Vec<Import>,
    pub calls: Vec<Call>,
    pub heritage: Vec<Heritage>,
}

