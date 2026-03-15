//! Codeilus database layer: pool, migrations, batch writer, and repositories.

pub mod batch_writer;
pub mod migrations;
pub mod pool;
pub mod repos;

pub use batch_writer::BatchWriter;
pub use migrations::Migrator;
pub use pool::DbPool;
pub use repos::{
    ChapterRepo, ChapterRow, ChapterSectionRow, CommunityRepo, CommunityRow, EdgeRepo, EdgeRow,
    FileMetricsRepo, FileMetricsRow, FileRepo, FileRow, HarvestRepoRepo, HarvestRepoRow,
    LearnerStatsRow, NarrativeRepo, NarrativeRow, PatternRepo, PatternRow, ProcessRepo,
    ProcessRow, ProcessStepRow, ProgressRepo, ProgressRow, QuizQuestionRow, QuizRepo, SymbolRepo,
    SymbolRow,
};

use std::collections::HashMap;

use codeilus_core::error::CodeilusResult;
use codeilus_core::CodeilusError;
use codeilus_parse::ParsedFile;
use repos::files::NewFile;
use repos::symbols::NewSymbol;

impl DbPool {
    /// Delete all analysis data in FK-safe order, enabling re-analysis.
    pub fn clear_analysis_data(&self) -> CodeilusResult<()> {
        let conn_arc = self.conn_arc();
        QuizRepo::new(conn_arc.clone()).delete_all()?;
        ProgressRepo::new(conn_arc.clone()).delete_all()?;
        ChapterRepo::new(conn_arc.clone()).delete_all()?;
        ProcessRepo::new(conn_arc.clone()).delete_all()?;
        CommunityRepo::new(conn_arc.clone()).delete_all()?;
        NarrativeRepo::new(conn_arc.clone()).delete_all()?;
        PatternRepo::new(conn_arc.clone()).delete_all()?;
        FileMetricsRepo::new(conn_arc.clone()).delete_all()?;
        EdgeRepo::new(conn_arc.clone()).delete_all()?;
        SymbolRepo::new(conn_arc.clone()).delete_all()?;
        FileRepo::new(conn_arc).delete_all()?;
        Ok(())
    }

    /// Persist a collection of parsed files into the database.
    ///
    /// For Sprint 1 this inserts rows into `files` and `symbols`. Edge persistence
    /// is intentionally deferred until we have more robust resolution in Sprint 2.
    pub fn persist_parsed_files(&self, parsed: &[ParsedFile]) -> Result<(), CodeilusError> {
        let mut conn = self.connection();
        let tx = conn
            .transaction()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        let file_repo = repos::files::FileRepo;
        let symbol_repo = repos::symbols::SymbolRepo;

        // Insert files.
        let new_files: Vec<NewFile> = parsed
            .iter()
            .map(|pf| NewFile {
                path: pf.path.to_string_lossy().to_string(),
                language: pf.language,
                sloc: Some(pf.sloc as i64),
                last_modified: None,
            })
            .collect();

        let file_ids = file_repo.insert_files(&tx, &new_files)?;

        // Map file paths to their assigned IDs.
        let path_to_id: HashMap<String, _> = new_files
            .iter()
            .zip(file_ids.iter())
            .map(|(file, id)| (file.path.clone(), *id))
            .collect();

        // Flatten symbols across all files.
        let mut all_symbols = Vec::new();
        for pf in parsed {
            let path = pf.path.to_string_lossy().to_string();
            if let Some(file_id) = path_to_id.get(&path) {
                for sym in &pf.symbols {
                    all_symbols.push(NewSymbol {
                        file_id: *file_id,
                        name: sym.name.clone(),
                        kind: sym.kind,
                        start_line: sym.start_line,
                        end_line: sym.end_line,
                        signature: sym.signature.clone(),
                    });
                }
            }
        }

        if !all_symbols.is_empty() {
            let _symbol_ids = symbol_repo.insert_symbols(&tx, &all_symbols)?;
        }

        tx.commit()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }
}

