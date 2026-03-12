//! Symbol API routes.

use axum::extract::{Path, Query, State};
use axum::routing::get;
use axum::{Json, Router};
use serde::Deserialize;

use codeilus_core::error::CodeilusError;
use codeilus_core::ids::{FileId, SymbolId};
use codeilus_db::{SymbolRepo, SymbolRow};
use rusqlite::params;

use crate::error::ApiError;
use crate::state::AppState;

#[derive(Deserialize)]
pub struct SymbolSearchQuery {
    pub q: Option<String>,
    pub kind: Option<String>,
}

/// GET /api/v1/symbols — List all symbols, optional ?kind= filter
async fn list_symbols(
    State(state): State<AppState>,
    Query(query): Query<SymbolSearchQuery>,
) -> Result<Json<Vec<SymbolRow>>, ApiError> {
    let conn = state.db.connection();
    let mut result = Vec::new();
    match query.kind {
        Some(kind) => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, file_id, name, kind, start_line, end_line, signature FROM symbols WHERE kind = ?1",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            let rows = stmt
                .query_map(params![kind], |row| {
                    Ok(SymbolRow {
                        id: SymbolId(row.get(0)?),
                        file_id: FileId(row.get(1)?),
                        name: row.get(2)?,
                        kind: row.get(3)?,
                        start_line: row.get(4)?,
                        end_line: row.get(5)?,
                        signature: row.get(6)?,
                    })
                })
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for row in rows {
                result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
            }
        }
        None => {
            let mut stmt = conn
                .prepare(
                    "SELECT id, file_id, name, kind, start_line, end_line, signature FROM symbols",
                )
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            let rows = stmt
                .query_map([], |row| {
                    Ok(SymbolRow {
                        id: SymbolId(row.get(0)?),
                        file_id: FileId(row.get(1)?),
                        name: row.get(2)?,
                        kind: row.get(3)?,
                        start_line: row.get(4)?,
                        end_line: row.get(5)?,
                        signature: row.get(6)?,
                    })
                })
                .map_err(|e| CodeilusError::Database(Box::new(e)))?;
            for row in rows {
                result.push(row.map_err(|e| CodeilusError::Database(Box::new(e)))?);
            }
        }
    }
    Ok(Json(result))
}

/// GET /api/v1/symbols/:id — Get a single symbol by ID
async fn get_symbol(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<SymbolRow>, ApiError> {
    let repo = SymbolRepo::new(state.db.conn_arc());
    let symbol = repo.get(SymbolId(id))?;
    Ok(Json(symbol))
}

/// GET /api/v1/symbols/search?q=foo — Search symbols by name prefix
async fn search_symbols(
    State(state): State<AppState>,
    Query(query): Query<SymbolSearchQuery>,
) -> Result<Json<Vec<SymbolRow>>, ApiError> {
    let repo = SymbolRepo::new(state.db.conn_arc());
    let q = query.q.as_deref().unwrap_or("");
    let results = repo.search(q)?;
    Ok(Json(results))
}

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/symbols", get(list_symbols))
        .route("/symbols/search", get(search_symbols))
        .route("/symbols/:id", get(get_symbol))
}
