//! API route definitions.

pub mod health;
pub mod ws;

use axum::Router;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .merge(ws::routes())
}
