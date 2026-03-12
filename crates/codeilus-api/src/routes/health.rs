//! Health check endpoint.

use axum::{routing::get, Json, Router};
use crate::state::AppState;
use serde_json::{json, Value};

pub fn routes() -> Router<AppState> {
    Router::new().route("/health", get(health_check))
}

async fn health_check() -> Json<Value> {
    Json(json!({ "status": "ok" }))
}
