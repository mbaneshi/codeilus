use std::path::Path;

use codeilus_core::{CodeilusError, Language};

use crate::model::ParsedFile;

mod python;
mod typescript;
mod javascript;
mod rustlang;
mod golang;
mod javalang;

pub fn parse_file(
    path: &Path,
    language: Language,
    source: &str,
) -> Result<ParsedFile, CodeilusError> {
    match language {
        Language::Python => python::parse(path, source),
        Language::TypeScript => typescript::parse(path, source),
        Language::JavaScript => javascript::parse(path, source),
        Language::Rust => rustlang::parse(path, source),
        Language::Go => golang::parse(path, source),
        Language::Java => javalang::parse(path, source),
        _ => Err(CodeilusError::Parse(format!(
            "Unsupported language for parse_file: {:?}",
            language
        ))),
    }
}

