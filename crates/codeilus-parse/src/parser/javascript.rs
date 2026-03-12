use std::path::Path;

use codeilus_core::{CodeilusError, Language, SymbolKind};

use crate::model::{Call, Heritage, Import, ParsedFile, Symbol};

pub fn parse(path: &Path, source: &str) -> Result<ParsedFile, CodeilusError> {
    let mut symbols = Vec::new();
    let mut imports = Vec::new();

    for (idx, line) in source.lines().enumerate() {
        let line_no = (idx + 1) as i64;
        let trimmed = line.trim_start();

        if trimmed.starts_with("function ") {
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
                let cls_name = name.split('{').next().unwrap_or(name).to_string();
                symbols.push(Symbol {
                    name: cls_name,
                    kind: SymbolKind::Class,
                    start_line: line_no,
                    end_line: line_no,
                    signature: Some(trimmed.to_string()),
                });
            }
        } else if trimmed.starts_with("import ") {
            // import X from 'module';
            if let Some(from_idx) = trimmed.find(" from ") {
                let names_part = &trimmed["import ".len()..from_idx];
                let module_part = &trimmed[from_idx + " from ".len()..];
                let module = module_part.trim().trim_matches(|c| c == '"' || c == '\'');
                imports.push(Import {
                    from: module.to_string(),
                    name: names_part.trim().to_string(),
                });
            }
        } else if trimmed.starts_with("require(") {
            // const X = require('module')
            if let Some(start) = trimmed.find("require(") {
                let rest = &trimmed[start + "require(".len()..];
                if let Some(end) = rest.find(')') {
                    let module = &rest[..end];
                    let module = module.trim().trim_matches(|c| c == '"' || c == '\'');
                    imports.push(Import {
                        from: module.to_string(),
                        name: "*".to_string(),
                    });
                }
            }
        }
    }

    Ok(ParsedFile {
        path: path.to_path_buf(),
        language: Language::JavaScript,
        symbols,
        imports,
        calls: Vec::new(),
        heritage: Vec::new(),
    })
}

