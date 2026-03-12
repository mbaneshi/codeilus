use std::path::Path;

use codeilus_core::{Confidence, EdgeKind, Language, SymbolKind};
use streaming_iterator::StreamingIterator;
use tree_sitter::{Query, QueryCursor, Tree};

use crate::model::{Call, Heritage, Import, ParsedFile, Symbol};
use crate::queries::{self, LanguageQueries};

/// Count source lines of code (non-empty lines).
fn count_sloc(source: &str) -> usize {
    source.lines().filter(|line| !line.trim().is_empty()).count()
}

/// Extract all symbols, imports, calls, and heritage from a parsed tree.
pub fn extract(tree: &Tree, source: &[u8], lang: Language, path: &Path) -> ParsedFile {
    let queries = queries::get_queries(lang);
    let ts_lang = tree.language();

    let symbols = extract_definitions(tree, source, &ts_lang, queries, lang);
    let imports = extract_imports(tree, source, &ts_lang, queries);
    let calls = extract_calls(tree, source, &ts_lang, queries, &symbols);
    let heritage = extract_heritage(tree, source, &ts_lang, queries, lang);
    let sloc = count_sloc(std::str::from_utf8(source).unwrap_or(""));

    ParsedFile {
        path: path.to_path_buf(),
        language: lang,
        sloc,
        symbols,
        imports,
        calls,
        heritage,
    }
}

fn extract_definitions(
    tree: &Tree,
    source: &[u8],
    ts_lang: &tree_sitter::Language,
    queries: &LanguageQueries,
    lang: Language,
) -> Vec<Symbol> {
    let mut symbols = Vec::new();

    let query = match Query::new(ts_lang, queries.definitions) {
        Ok(q) => q,
        Err(e) => {
            tracing::warn!("Failed to compile definitions query for {lang}: {e}");
            return symbols;
        }
    };

    let name_idx = match query.capture_index_for_name("name") {
        Some(idx) => idx,
        None => return symbols,
    };
    let def_idx = query.capture_index_for_name("def");

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source);

    let mut seen: Vec<(String, i64)> = Vec::new();

    while let Some(m) = matches.next() {
        let mut name = String::new();
        let mut start_line: i64 = 0;
        let mut end_line: i64 = 0;
        let mut node_kind = "";

        for capture in m.captures {
            if capture.index == name_idx {
                name = node_text(capture.node, source).to_string();
            }
            if let Some(di) = def_idx {
                if capture.index == di {
                    start_line = capture.node.start_position().row as i64 + 1;
                    end_line = capture.node.end_position().row as i64 + 1;
                    node_kind = capture.node.kind();
                }
            }
        }

        if name.is_empty() {
            continue;
        }

        // If no @def capture matched, use the @name node's parent
        if start_line == 0 {
            for capture in m.captures {
                if capture.index == name_idx {
                    if let Some(parent) = capture.node.parent() {
                        start_line = parent.start_position().row as i64 + 1;
                        end_line = parent.end_position().row as i64 + 1;
                        node_kind = parent.kind();
                    } else {
                        start_line = capture.node.start_position().row as i64 + 1;
                        end_line = capture.node.end_position().row as i64 + 1;
                    }
                }
            }
        }

        // Dedup
        let key = (name.clone(), start_line);
        if seen.contains(&key) {
            continue;
        }
        seen.push(key);

        let kind = classify_symbol(node_kind, lang);
        let signature = build_signature(source, start_line);

        symbols.push(Symbol {
            name,
            kind,
            start_line,
            end_line,
            signature,
        });
    }

    symbols
}

fn classify_symbol(node_kind: &str, lang: Language) -> SymbolKind {
    match node_kind {
        "function_definition" | "function_declaration" | "function_item" => SymbolKind::Function,
        "class_definition" | "class_declaration" => SymbolKind::Class,
        "method_definition" | "method_declaration" => SymbolKind::Method,
        "interface_declaration" => SymbolKind::Interface,
        "struct_item" => SymbolKind::Struct,
        "enum_item" => SymbolKind::Enum,
        "trait_item" => SymbolKind::Trait,
        "type_declaration" | "type_spec" => {
            if lang == Language::Go {
                SymbolKind::Struct
            } else {
                SymbolKind::TypeAlias
            }
        }
        "impl_item" => SymbolKind::Method,
        _ => SymbolKind::Function,
    }
}

fn build_signature(source: &[u8], start_line: i64) -> Option<String> {
    let source_str = std::str::from_utf8(source).ok()?;
    let line = source_str.lines().nth((start_line - 1) as usize)?;
    Some(line.trim().to_string())
}

fn extract_imports(
    tree: &Tree,
    source: &[u8],
    ts_lang: &tree_sitter::Language,
    queries: &LanguageQueries,
) -> Vec<Import> {
    let mut imports = Vec::new();

    if queries.imports.trim().is_empty() {
        return imports;
    }

    let query = match Query::new(ts_lang, queries.imports) {
        Ok(q) => q,
        Err(e) => {
            tracing::warn!("Failed to compile imports query: {e}");
            return imports;
        }
    };

    let module_idx = query.capture_index_for_name("module");
    let name_idx = query.capture_index_for_name("name");

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source);

    while let Some(m) = matches.next() {
        let mut module = String::new();
        let mut name = String::new();
        let mut line: i64 = 0;

        for capture in m.captures {
            if let Some(mi) = module_idx {
                if capture.index == mi {
                    module = node_text(capture.node, source)
                        .trim_matches(|c: char| c == '"' || c == '\'' || c == '`' || c == ';')
                        .to_string();
                    line = capture.node.start_position().row as i64 + 1;
                }
            }
            if let Some(ni) = name_idx {
                if capture.index == ni {
                    name = node_text(capture.node, source).to_string();
                }
            }
        }

        if !module.is_empty() {
            imports.push(Import {
                from: module,
                name: if name.is_empty() {
                    "*".to_string()
                } else {
                    name
                },
                line,
            });
        }
    }

    imports
}

fn extract_calls(
    tree: &Tree,
    source: &[u8],
    ts_lang: &tree_sitter::Language,
    queries: &LanguageQueries,
    symbols: &[Symbol],
) -> Vec<Call> {
    let mut calls = Vec::new();

    if queries.calls.trim().is_empty() {
        return calls;
    }

    let query = match Query::new(ts_lang, queries.calls) {
        Ok(q) => q,
        Err(e) => {
            tracing::warn!("Failed to compile calls query: {e}");
            return calls;
        }
    };

    let callee_idx = match query.capture_index_for_name("callee") {
        Some(idx) => idx,
        None => return calls,
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source);

    while let Some(m) = matches.next() {
        for capture in m.captures {
            if capture.index == callee_idx {
                let callee = node_text(capture.node, source).to_string();
                let line = capture.node.start_position().row as i64 + 1;
                let caller = find_enclosing_symbol(symbols, line);

                calls.push(Call {
                    caller: caller.unwrap_or_else(|| "<module>".to_string()),
                    callee,
                    line,
                });
            }
        }
    }

    calls
}

fn find_enclosing_symbol(symbols: &[Symbol], line: i64) -> Option<String> {
    symbols
        .iter()
        .filter(|s| s.start_line <= line && s.end_line >= line)
        .min_by_key(|s| s.end_line - s.start_line)
        .map(|s| s.name.clone())
}

fn extract_heritage(
    tree: &Tree,
    source: &[u8],
    ts_lang: &tree_sitter::Language,
    queries: &LanguageQueries,
    lang: Language,
) -> Vec<Heritage> {
    let mut heritage = Vec::new();

    if queries.heritage.trim().is_empty() {
        return heritage;
    }

    let query = match Query::new(ts_lang, queries.heritage) {
        Ok(q) => q,
        Err(e) => {
            tracing::warn!("Failed to compile heritage query for {lang}: {e}");
            return heritage;
        }
    };

    let child_idx = query.capture_index_for_name("child");
    let parent_idx = query.capture_index_for_name("parent");

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source);

    while let Some(m) = matches.next() {
        let mut child = String::new();
        let mut parent = String::new();

        for capture in m.captures {
            if let Some(ci) = child_idx {
                if capture.index == ci {
                    child = node_text(capture.node, source).to_string();
                }
            }
            if let Some(pi) = parent_idx {
                if capture.index == pi {
                    parent = node_text(capture.node, source).to_string();
                }
            }
        }

        if !child.is_empty() && !parent.is_empty() {
            let (relation, confidence) = classify_heritage(lang);
            heritage.push(Heritage {
                child,
                parent,
                relation,
                confidence,
            });
        }
    }

    heritage
}

fn classify_heritage(lang: Language) -> (EdgeKind, Confidence) {
    match lang {
        Language::Python => (EdgeKind::Extends, Confidence::certain()),
        Language::Java => (EdgeKind::Extends, Confidence::high()),
        Language::TypeScript | Language::JavaScript => (EdgeKind::Extends, Confidence::high()),
        Language::Rust => (EdgeKind::Implements, Confidence::certain()),
        _ => (EdgeKind::Extends, Confidence::medium()),
    }
}

fn node_text<'a>(node: tree_sitter::Node, source: &'a [u8]) -> &'a str {
    node.utf8_text(source).unwrap_or("")
}
