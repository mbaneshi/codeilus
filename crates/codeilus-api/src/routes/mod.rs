//! API route definitions.

pub mod ask;
pub mod chapters;
pub mod files;
pub mod graph;
pub mod health;
pub mod narratives;
pub mod search;
pub mod symbols;
pub mod ws;

use axum::Router;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .merge(health::routes())
        .merge(ws::routes())
        .merge(files::router())
        .merge(symbols::router())
        .merge(graph::router())
        .merge(search::router())
        .merge(ask::router())
        .merge(narratives::router())
        .merge(chapters::router())
}
