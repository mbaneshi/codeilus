use std::path::Path;

use codeilus_core::CodeilusError;

use crate::extractor;
use crate::language;
use crate::model::ParsedFile;

pub fn parse(path: &Path, source: &str) -> Result<ParsedFile, CodeilusError> {
    let mut parser = language::create_parser(codeilus_core::Language::JavaScript)?;
    let tree = parser
        .parse(source.as_bytes(), None)
        .ok_or_else(|| CodeilusError::Parse("Failed to parse JavaScript file".to_string()))?;

    Ok(extractor::extract(
        &tree,
        source.as_bytes(),
        codeilus_core::Language::JavaScript,
        path,
    ))
}
