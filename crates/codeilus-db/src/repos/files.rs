use chrono::{DateTime, Utc};
use rusqlite::{params, Transaction};

use codeilus_core::{CodeilusError, FileId, Language};

pub struct NewFile {
    pub path: String,
    pub language: Language,
    pub sloc: Option<i64>,
    pub last_modified: Option<DateTime<Utc>>,
}

pub struct FileRepo;

impl FileRepo {
    pub fn insert_files(
        &self,
        tx: &Transaction,
        files: &[NewFile],
    ) -> Result<Vec<FileId>, CodeilusError> {
        let mut stmt = tx
            .prepare("INSERT INTO files (path, language, sloc, last_modified) VALUES (?1, ?2, ?3, ?4)")
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        let mut ids = Vec::with_capacity(files.len());
        for file in files {
            stmt.execute(params![
                file.path,
                file.language.to_string(),
                file.sloc,
                file.last_modified.map(|dt| dt.to_rfc3339()),
            ])
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            let id = tx.last_insert_rowid();
            ids.push(FileId(id));
        }
        Ok(ids)
    }
}

