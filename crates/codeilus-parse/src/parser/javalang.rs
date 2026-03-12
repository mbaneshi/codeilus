use std::path::Path;

use codeilus_core::{CodeilusError, Language, SymbolKind};

use crate::model::{Call, Heritage, Import, ParsedFile, Symbol};

pub fn parse(path: &Path, source: &str) -> Result<ParsedFile, CodeilusError> {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        let line_no = (idx + 1) as i64;
        let trimmed = line.trim_start();

        if trimmed.starts_with("class ") || trimmed.contains(" class ") {
            if let Some(name) = trimmed.split_whitespace().nth(1) {
                let cls_name = name.trim_end_matches('{').to_string();
                symbols.push(Symbol {
                    name: cls_name,
                    kind: SymbolKind::Class,
                    start_line: line_no,
                    end_line: line_no,
                    signature: Some(trimmed.to_string()),
                });
            }
        } else if trimmed.starts_with("interface ") {
            if let Some(name) = trimmed.split_whitespace().nth(1) {
                let iface_name = name.trim_end_matches('{').to_string();
                symbols.push(Symbol {
                    name: iface_name,
                    kind: SymbolKind::Interface,
                    start_line: line_no,
                    end_line: line_no,
                    signature: Some(trimmed.to_string()),
                });
            }
        } else if trimmed.starts_with("import ") {
            let rest = trimmed.strip_prefix("import ").unwrap_or("").trim_end_matches(';').trim();
            imports.push(Import {
                from: rest.to_string(),
                name: "*".to_string(),
            });
        }
    }

    Ok(ParsedFile {
        path: path.to_path_buf(),
        language: Language::Java,
        symbols,
        imports,
        calls: Vec::new(),
        heritage: Vec::new(),
    })
}

