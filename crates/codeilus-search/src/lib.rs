//! BM25 full-text search using SQLite FTS5 with RRF ranking.

use std::sync::Arc;

use codeilus_core::error::CodeilusError;
use codeilus_db::DbPool;
use serde::{Deserialize, Serialize};
use tracing::debug;

/// A single search result from any FTS table.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: i64,
    pub kind: SearchResultKind,
    pub name: String,
    pub snippet: String,
    pub score: f64,
    pub metadata: SearchMetadata,
}

/// Which table the result originated from.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchResultKind {
    File,
    Symbol,
    Narrative,
}

/// Extra fields depending on result kind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub language: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub symbol_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line_range: Option<(i64, i64)>,
}

/// BM25 search engine backed by SQLite FTS5.
pub struct SearchEngine {
    db: Arc<DbPool>,
}

impl SearchEngine {
    /// Create a new search engine using the given database pool.
    pub fn new(db: Arc<DbPool>) -> Self {
        Self { db }
    }

    /// Unified search across files, symbols, and narratives.
    ///
    /// If `kind` is `Some`, only that table is searched. Otherwise all three are
    /// queried and results are merged using Reciprocal Rank Fusion (RRF).
    pub fn search(
        &self,
        q: &str,
        kind: Option<SearchResultKind>,
        limit: usize,
    ) -> Result<Vec<SearchResult>, CodeilusError> {
        let query = sanitize_fts_query(q);
        if query.is_empty() {
            return Ok(Vec::new());
        }

        match kind {
            Some(SearchResultKind::File) => self.search_files(&query, limit),
            Some(SearchResultKind::Symbol) => self.search_symbols(&query, limit),
            Some(SearchResultKind::Narrative) => self.search_narratives(&query, limit),
            None => self.search_unified(&query, limit),
        }
    }

    /// Search only the files FTS index.
    pub fn search_files(
        &self,
        q: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, CodeilusError> {
        let query = sanitize_fts_query(q);
        if query.is_empty() {
            return Ok(Vec::new());
        }
        debug!(query = %query, "searching files_fts");

        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT f.id, f.path, f.language, rank
                 FROM files_fts
                 JOIN files f ON f.id = files_fts.rowid
                 WHERE files_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        let results = stmt
            .query_map(rusqlite::params![query, limit as i64], |row| {
                let id: i64 = row.get(0)?;
                let path: String = row.get(1)?;
                let language: Option<String> = row.get(2)?;
                let rank: f64 = row.get(3)?;
                Ok(SearchResult {
                    id,
                    kind: SearchResultKind::File,
                    name: path.clone(),
                    snippet: path.clone(),
                    score: -rank, // FTS5 rank is negative; negate for display
                    metadata: SearchMetadata {
                        language,
                        file_path: Some(path),
                        symbol_kind: None,
                        line_range: None,
                    },
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        Ok(results)
    }

    /// Search only the symbols FTS index.
    pub fn search_symbols(
        &self,
        q: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, CodeilusError> {
        let query = sanitize_fts_query(q);
        if query.is_empty() {
            return Ok(Vec::new());
        }
        debug!(query = %query, "searching symbols_fts");

        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT s.id, s.name, s.kind, s.signature, s.start_line, s.end_line,
                        f.path, f.language, rank
                 FROM symbols_fts
                 JOIN symbols s ON s.id = symbols_fts.rowid
                 JOIN files f ON f.id = s.file_id
                 WHERE symbols_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        let results = stmt
            .query_map(rusqlite::params![query, limit as i64], |row| {
                let id: i64 = row.get(0)?;
                let name: String = row.get(1)?;
                let sym_kind: String = row.get(2)?;
                let signature: Option<String> = row.get(3)?;
                let start_line: Option<i64> = row.get(4)?;
                let end_line: Option<i64> = row.get(5)?;
                let file_path: String = row.get(6)?;
                let language: Option<String> = row.get(7)?;
                let rank: f64 = row.get(8)?;

                let snippet = signature.unwrap_or_else(|| name.clone());
                let line_range = match (start_line, end_line) {
                    (Some(s), Some(e)) => Some((s, e)),
                    _ => None,
                };

                Ok(SearchResult {
                    id,
                    kind: SearchResultKind::Symbol,
                    name,
                    snippet,
                    score: -rank,
                    metadata: SearchMetadata {
                        language,
                        file_path: Some(file_path),
                        symbol_kind: Some(sym_kind),
                        line_range,
                    },
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        Ok(results)
    }

    /// Search only the narratives FTS index.
    pub fn search_narratives(
        &self,
        q: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, CodeilusError> {
        let query = sanitize_fts_query(q);
        if query.is_empty() {
            return Ok(Vec::new());
        }
        debug!(query = %query, "searching narratives_fts");

        let conn = self.db.connection();
        let mut stmt = conn
            .prepare(
                "SELECT n.id, n.kind, n.content, rank
                 FROM narratives_fts
                 JOIN narratives n ON n.id = narratives_fts.rowid
                 WHERE narratives_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        let results = stmt
            .query_map(rusqlite::params![query, limit as i64], |row| {
                let id: i64 = row.get(0)?;
                let kind: String = row.get(1)?;
                let content: String = row.get(2)?;
                let rank: f64 = row.get(3)?;

                // Truncate content for snippet
                let snippet = if content.len() > 200 {
                    format!("{}...", &content[..200])
                } else {
                    content
                };

                Ok(SearchResult {
                    id,
                    kind: SearchResultKind::Narrative,
                    name: kind.clone(),
                    snippet,
                    score: -rank,
                    metadata: SearchMetadata {
                        language: None,
                        file_path: None,
                        symbol_kind: None,
                        line_range: None,
                    },
                })
            })
            .map_err(|e| CodeilusError::Database(Box::new(e)))?
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| CodeilusError::Database(Box::new(e)))?;

        Ok(results)
    }

    /// Drop and recreate FTS content from the source tables.
    pub fn rebuild_index(&self) -> Result<(), CodeilusError> {
        debug!("rebuilding FTS indexes");
        let conn = self.db.connection();
        conn.execute_batch(
            "DELETE FROM files_fts;
             DELETE FROM symbols_fts;
             DELETE FROM narratives_fts;
             INSERT INTO files_fts(rowid, path, language) SELECT id, path, language FROM files;
             INSERT INTO symbols_fts(rowid, name, kind, signature) SELECT id, name, kind, signature FROM symbols;
             INSERT INTO narratives_fts(rowid, kind, content) SELECT id, kind, content FROM narratives;",
        )
        .map_err(|e| CodeilusError::Database(Box::new(e)))?;
        Ok(())
    }

    /// Unified search: query all three FTS tables and merge via RRF.
    fn search_unified(
        &self,
        query: &str,
        limit: usize,
    ) -> Result<Vec<SearchResult>, CodeilusError> {
        // Fetch from each table with a generous per-table limit
        let per_table_limit = limit;
        let mut file_results = self.search_files(query, per_table_limit)?;
        let mut symbol_results = self.search_symbols(query, per_table_limit)?;
        let mut narrative_results = self.search_narratives(query, per_table_limit)?;

        // Apply RRF scoring based on rank position within each list
        apply_rrf_scores(&mut file_results);
        apply_rrf_scores(&mut symbol_results);
        apply_rrf_scores(&mut narrative_results);

        // Combine and sort by RRF score descending
        let mut combined = Vec::with_capacity(
            file_results.len() + symbol_results.len() + narrative_results.len(),
        );
        combined.append(&mut file_results);
        combined.append(&mut symbol_results);
        combined.append(&mut narrative_results);
        combined.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        combined.truncate(limit);

        Ok(combined)
    }
}

/// Apply Reciprocal Rank Fusion scores: `score = 1.0 / (60.0 + rank_position)`.
fn apply_rrf_scores(results: &mut [SearchResult]) {
    for (i, result) in results.iter_mut().enumerate() {
        result.score = 1.0 / (60.0 + i as f64);
    }
}

/// Sanitize user input for FTS5 MATCH syntax.
///
/// Each word is wrapped in double quotes to prevent FTS5 syntax errors from
/// special characters like `*`, `-`, `OR`, etc.
fn sanitize_fts_query(input: &str) -> String {
    let words: Vec<String> = input
        .split_whitespace()
        .map(|w| {
            // Strip existing quotes and FTS5 special chars
            let cleaned: String = w
                .chars()
                .filter(|c| c.is_alphanumeric() || *c == '_' || *c == '.')
                .collect();
            cleaned
        })
        .filter(|w| !w.is_empty())
        .map(|w| format!("\"{}\"", w))
        .collect();
    words.join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sanitize_basic() {
        assert_eq!(sanitize_fts_query("hello world"), "\"hello\" \"world\"");
    }

    #[test]
    fn test_sanitize_special_chars() {
        assert_eq!(sanitize_fts_query("foo* -bar OR baz"), "\"foo\" \"bar\" \"OR\" \"baz\"");
    }

    #[test]
    fn test_sanitize_empty() {
        assert_eq!(sanitize_fts_query(""), "");
        assert_eq!(sanitize_fts_query("   "), "");
        assert_eq!(sanitize_fts_query("***"), "");
    }

    #[test]
    fn test_sanitize_quotes() {
        assert_eq!(sanitize_fts_query("\"hello\""), "\"hello\"");
    }
}
