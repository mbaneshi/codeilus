//! Tree-sitter multi-language parsing and symbol extraction.

mod extractor;
mod language;
mod model;
mod parser;
pub mod queries;
pub mod resolver;
mod walker;

use std::path::PathBuf;

use codeilus_core::{CodeilusError, CodeilusEvent, EventBus};

pub use language::{create_parser, detect_language};
pub use model::{Call, Heritage, Import, ParsedFile, Symbol};

/// Configuration for repository parsing.
pub struct ParseConfig {
    pub root: PathBuf,
    pub follow_symlinks: bool,
    pub max_file_bytes: usize,
}

impl ParseConfig {
    pub fn new(root: PathBuf) -> Self {
        Self {
            root,
            follow_symlinks: false,
            max_file_bytes: 20 * 1024 * 1024, // 20MB default
        }
    }
}

/// Parse an entire repository into `ParsedFile` records.
///
/// If an `EventBus` is provided, emits progress events as files are parsed.
pub fn parse_repository(
    config: &ParseConfig,
    bus: Option<&EventBus>,
) -> Result<Vec<ParsedFile>, CodeilusError> {
    use rayon::prelude::*;

    let files = walker::walk_files(&config.root, config.follow_symlinks, config.max_file_bytes)?;
    let total = files.len();

    if let Some(bus) = bus {
        let path = config.root.to_string_lossy().to_string();
        bus.publish(CodeilusEvent::AnalysisStarted { path });
    }

    let parsed_files: Vec<ParsedFile> = files
        .par_iter()
        .filter_map(|path| {
            let lang = language::detect_language(path)?;
            let source = std::fs::read_to_string(path).ok()?;
            parser::parse_file(path.as_path(), lang, &source).ok()
        })
        .collect();

    if let Some(bus) = bus {
        for (idx, _) in parsed_files.iter().enumerate() {
            bus.publish(CodeilusEvent::ParsingProgress {
                files_done: idx + 1,
                files_total: total,
            });
        }
        let files_count = parsed_files.len();
        let symbols: usize = parsed_files.iter().map(|pf| pf.symbols.len()).sum();
        bus.publish(CodeilusEvent::ParsingComplete {
            files: files_count,
            symbols,
        });
    }

    Ok(parsed_files)
}
