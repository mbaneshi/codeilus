use std::path::Path;

use codeilus_core::{CodeilusError, Language, SymbolKind};

use crate::model::{Call, Heritage, Import, ParsedFile, Symbol};

pub fn parse(path: &Path, source: &str) -> Result<ParsedFile, CodeilusError> {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        let line_no = (idx + 1) as i64;
        let trimmed = line.trim_start();

        if trimmed.starts_with("fn ") {
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
        } else if trimmed.starts_with("struct ") {
            if let Some(name) = trimmed.split_whitespace().nth(1) {
                symbols.push(Symbol {
                    name: name.to_string(),
                    kind: SymbolKind::Struct,
                    start_line: line_no,
                    end_line: line_no,
                    signature: Some(trimmed.to_string()),
                });
            }
        } else if trimmed.starts_with("trait ") {
            if let Some(name) = trimmed.split_whitespace().nth(1) {
                symbols.push(Symbol {
                    name: name.to_string(),
                    kind: SymbolKind::Trait,
                    start_line: line_no,
                    end_line: line_no,
                    signature: Some(trimmed.to_string()),
                });
            }
        } else if trimmed.starts_with("use ") {
            let rest = trimmed.strip_prefix("use ").unwrap_or("").trim_end_matches(';').trim();
            imports.push(Import {
                from: rest.to_string(),
                name: "*".to_string(),
            });
        }
    }

    Ok(ParsedFile {
        path: path.to_path_buf(),
        language: Language::Rust,
        symbols,
        imports,
        calls: Vec::new(),
        heritage: Vec::new(),
    })
}

