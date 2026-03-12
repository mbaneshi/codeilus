use std::path::Path;

use codeilus_core::{CodeilusError, Language, SymbolKind};

use crate::model::{Call, Heritage, Import, ParsedFile, Symbol};

pub fn parse(path: &Path, source: &str) -> Result<ParsedFile, CodeilusError> {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        let line_no = (idx + 1) as i64;
        let trimmed = line.trim_start();

        if trimmed.starts_with("func ") {
            // func Name(...
            if let Some(name) = trimmed.split_whitespace().nth(1) {
                let func_name = name.split('(').next().unwrap_or(name).to_string();
                symbols.push(Symbol {
                    name: func_name,
                    kind: SymbolKind::Function,
                    start_line: line_no,
                    end_line: line_no,
                    signature: Some(trimmed.to_string()),
                });
            }
        } else if trimmed.starts_with("type ") && trimmed.contains(" struct") {
            if let Some(name) = trimmed.split_whitespace().nth(1) {
                symbols.push(Symbol {
                    name: name.to_string(),
                    kind: SymbolKind::Struct,
                    start_line: line_no,
                    end_line: line_no,
                    signature: Some(trimmed.to_string()),
                });
            }
        } else if trimmed.starts_with("import ") {
            // import "x" or import ( ... )
            if trimmed.contains('(') {
                // multi-line imports not fully handled in Sprint 1; skip
            } else {
                let part = trimmed.strip_prefix("import ").unwrap_or("").trim();
                let module = part.trim_matches(|c| c == '"' || c == '`');
                imports.push(Import {
                    from: module.to_string(),
                    name: "*".to_string(),
                });
            }
        }
    }

    Ok(ParsedFile {
        path: path.to_path_buf(),
        language: Language::Go,
        symbols,
        imports,
        calls: Vec::new(),
        heritage: Vec::new(),
    })
}

