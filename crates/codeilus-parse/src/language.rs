use std::path::Path;

use codeilus_core::{CodeilusError, Language};
use tree_sitter::Parser;

pub fn detect_language(path: &Path) -> Option<Language> {
    let ext = path.extension()?.to_str()?.to_ascii_lowercase();
    Language::from_extension(&ext)
}

pub fn create_parser(lang: Language) -> Result<Parser, CodeilusError> {
    let mut parser = Parser::new();
    let ts_lang = get_ts_language(lang)?;
    parser
        .set_language(&ts_lang)
        .map_err(|e| CodeilusError::Parse(format!("Failed to set language: {e}")))?;
    Ok(parser)
}

fn get_ts_language(lang: Language) -> Result<tree_sitter::Language, CodeilusError> {
    match lang {
        Language::Python => Ok(tree_sitter_python::LANGUAGE.into()),
        Language::TypeScript => Ok(tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into()),
        Language::JavaScript => Ok(tree_sitter_javascript::LANGUAGE.into()),
        Language::Rust => Ok(tree_sitter_rust::LANGUAGE.into()),
        Language::Go => Ok(tree_sitter_go::LANGUAGE.into()),
        Language::Java => Ok(tree_sitter_java::LANGUAGE.into()),
        _ => Err(CodeilusError::Parse(format!(
            "Unsupported language for tree-sitter: {lang}"
        ))),
    }
}
