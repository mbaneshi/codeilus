//! Health check endpoint with diagnostics.

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::state::AppState;

#[derive(Serialize)]
struct HealthResponse {
    status: &'static str,
    version: &'static str,
    db_ok: bool,
    llm_available: bool,
    files_analyzed: i64,
    symbols_count: i64,
}

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check(State(state): State<AppState>) -> Json<HealthResponse> {
    let (db_ok, files_analyzed, symbols_count) = {
        let conn = state.db.connection();
        let ok = conn.execute_batch("SELECT 1").is_ok();
        let files: i64 = conn
            .query_row("SELECT COUNT(*) FROM files", [], |r| r.get(0))
            .unwrap_or(0);
        let symbols: i64 = conn
            .query_row("SELECT COUNT(*) FROM symbols", [], |r| r.get(0))
            .unwrap_or(0);
        (ok, files, symbols)
    };

    let llm_available = state.llm.is_available().await;

    Json(HealthResponse {
        status: "ok",
        version: env!("CARGO_PKG_VERSION"),
        db_ok,
        llm_available,
        files_analyzed,
        symbols_count,
    })
}
