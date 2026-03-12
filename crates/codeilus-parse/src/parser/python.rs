use std::path::Path;

use codeilus_core::{CodeilusError, Language, SymbolKind};

use crate::model::{Call, Heritage, Import, ParsedFile, Symbol};

pub fn parse(path: &Path, source: &str) -> Result<ParsedFile, CodeilusError> {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();

    // Extremely simple, line-oriented heuristics for Sprint 1.
    for (idx, line) in source.lines().enumerate() {
        let line_no = (idx + 1) as i64;
        let trimmed = line.trim_start();

        if trimmed.starts_with("def ") {
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
        } else if trimmed.starts_with("class ") {
            if let Some(name) = trimmed.split_whitespace().nth(1) {
                let cls_name = name.split('(').next().unwrap_or(name).to_string();
                symbols.push(Symbol {
                    name: cls_name,
                    kind: SymbolKind::Class,
                    start_line: line_no,
                    end_line: line_no,
                    signature: Some(trimmed.to_string()),
                });
            }
        } else if trimmed.starts_with("import ") {
            let rest = trimmed.strip_prefix("import ").unwrap_or("").trim();
            imports.push(Import {
                from: rest.to_string(),
                name: "*".to_string(),
            });
        } else if trimmed.starts_with("from ") {
            // from x import y
            let mut parts = trimmed.split_whitespace();
            let _from_kw = parts.next();
            if let Some(module) = parts.next() {
                // skip "import"
                let _import_kw = parts.next();
                if let Some(name) = parts.next() {
                    imports.push(Import {
                        from: module.to_string(),
                        name: name.trim_end_matches(',').to_string(),
                    });
                }
            }
        }
    }

    Ok(ParsedFile {
        path: path.to_path_buf(),
        language: Language::Python,
        symbols,
        imports,
        calls: Vec::new(),
        heritage: Vec::new(),
    })
}

