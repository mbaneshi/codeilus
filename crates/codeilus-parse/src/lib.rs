//! Tree-sitter multi-language parsing and symbol extraction.

mod extractor;
mod language;
mod model;
mod parser;
pub mod queries;
pub mod resolver;
mod walker;

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::SystemTime;

use codeilus_core::{CodeilusError, CodeilusEvent, EventBus, FileId};

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

/// Describes a file already stored in the database.
pub struct ExistingFile {
    pub id: FileId,
    /// Last modification time recorded during previous analysis.
    pub last_modified: Option<SystemTime>,
}

/// Result of an incremental parse: only changed/new files are re-parsed.
pub struct IncrementalParseResult {
    /// Newly parsed files (new or modified since last analysis).
    pub changed_files: Vec<ParsedFile>,
    /// IDs of files in the DB that have not changed on disk.
    pub unchanged_ids: Vec<FileId>,
}

/// Parse only new or modified files in a repository.
///
/// `existing` maps relative file paths (as stored in the DB) to their
/// [`ExistingFile`] records. The caller is responsible for querying the
/// database and populating this map before invoking this function.
///
/// Returns an [`IncrementalParseResult`] so the caller can selectively update
/// only the changed rows in the database.
pub fn parse_repository_incremental(
    config: &ParseConfig,
    existing: &HashMap<String, ExistingFile>,
    bus: Option<&EventBus>,
) -> Result<IncrementalParseResult, CodeilusError> {
    use rayon::prelude::*;

    let files = walker::walk_files(&config.root, config.follow_symlinks, config.max_file_bytes)?;

    if let Some(bus) = bus {
        let path = config.root.to_string_lossy().to_string();
        bus.publish(CodeilusEvent::AnalysisStarted { path });
    }

    let mut to_parse = Vec::new();
    let mut unchanged_ids = Vec::new();

    for path in &files {
        let rel = path
            .strip_prefix(&config.root)
            .unwrap_or(path)
            .to_string_lossy()
            .to_string();

        if let Some(entry) = existing.get(&rel) {
            // Compare filesystem mtime with stored last_modified
            let fs_mtime = std::fs::metadata(path)
                .ok()
                .and_then(|m| m.modified().ok());

            let changed = match (fs_mtime, entry.last_modified) {
                (Some(fs), Some(db)) => fs > db,
                // If we can't determine either mtime, re-parse to be safe
                (None, _) => true,
                (_, None) => true,
            };

            if changed {
                to_parse.push(path.clone());
            } else {
                unchanged_ids.push(entry.id);
            }
        } else {
            // New file not in DB
            to_parse.push(path.clone());
        }
    }

    let total = to_parse.len();
    let changed_files: Vec<ParsedFile> = to_parse
        .par_iter()
        .filter_map(|path| {
            let lang = language::detect_language(path)?;
            let source = std::fs::read_to_string(path).ok()?;
            parser::parse_file(path.as_path(), lang, &source).ok()
        })
        .collect();

    if let Some(bus) = bus {
        for (idx, _) in changed_files.iter().enumerate() {
            bus.publish(CodeilusEvent::ParsingProgress {
                files_done: idx + 1,
                files_total: total,
            });
        }
        let files_count = changed_files.len();
        let symbols: usize = changed_files.iter().map(|pf| pf.symbols.len()).sum();
        bus.publish(CodeilusEvent::ParsingComplete {
            files: files_count,
            symbols,
        });
    }

    Ok(IncrementalParseResult {
        changed_files,
        unchanged_ids,
    })
}
