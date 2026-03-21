//! API route definitions.

pub mod annotations;
pub mod ask;
pub mod chapters;
pub mod files;
pub mod graph;
pub mod health;
pub mod learning;
pub mod narratives;
pub mod schematic;
pub mod search;
pub mod symbols;
pub mod ws;

use axum::Router;
use axum::middleware::{self, Next};
use axum::extract::Request;
use axum::response::Response;
use http::Method;
use serde::Deserialize;
use crate::state::AppState;

/// Shared pagination query parameters for list endpoints.
#[derive(Deserialize)]
pub struct PaginationQuery {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

impl PaginationQuery {
    /// Returns (limit, offset) clamped to safe defaults.
    pub fn resolve(&self) -> (i64, i64) {
        let limit = self.limit.unwrap_or(50).clamp(1, 200);
        let offset = self.offset.unwrap_or(0).max(0);
        (limit, offset)
    }
}

/// Middleware that adds Cache-Control headers to GET responses.
async fn cache_control_middleware(request: Request, next: Next) -> Response {
    let is_get = request.method() == Method::GET;
    let mut response = next.run(request).await;
    if is_get {
        response.headers_mut().insert(
            http::header::CACHE_CONTROL,
            http::HeaderValue::from_static("public, max-age=300, stale-while-revalidate=60"),
        );
    }
    response
}

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
        .merge(annotations::router())
        .merge(schematic::router())
        .merge(learning::router())
        .layer(middleware::from_fn(cache_control_middleware))
}
