//! Codeilus HTTP API: health, WebSocket event stream, embedded frontend.

pub mod error;
pub mod routes;
pub mod state;

pub use state::AppState;

use axum::{
    body::Body,
    extract::Request,
    response::{IntoResponse, Response},
    Router,
};
use http::{header::CONTENT_TYPE, Method, StatusCode};
use mime_guess::from_path;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing::info;

#[derive(rust_embed::RustEmbed)]
#[folder = "../../frontend/build"]
#[allow_missing = true]
struct FrontendAssets;

pub fn app(state: AppState) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE, Method::OPTIONS])
        .allow_headers([CONTENT_TYPE]);

    Router::new()
        .nest("/api/v1", routes::router())
        .fallback(serve_embedded_fallback)
        .layer(TraceLayer::new_for_http())
        .layer(cors)
        .with_state(state)
}

async fn serve_embedded_fallback(request: Request) -> Response {
    if request.method() != Method::GET {
        return (StatusCode::METHOD_NOT_ALLOWED, Body::empty()).into_response();
    }

    let path = request.uri().path().trim_start_matches('/');
    let path = if path.is_empty() { "index.html" } else { path };

    let candidates = [
        path.to_string(),
        format!("{}/index.html", path),
        format!("{}.html", path),
        "index.html".to_string(),
    ];

    for candidate in &candidates {
        if let Some(file) = FrontendAssets::get(candidate.as_str()) {
            let mime = from_path(candidate.as_str()).first_or_octet_stream();
            if let Ok(value) = http::HeaderValue::try_from(mime.as_ref()) {
                return ([(CONTENT_TYPE, value)], file.data.to_vec()).into_response();
            }
        }
    }

    if let Some(index) = FrontendAssets::get("index.html") {
        let value = http::HeaderValue::from_static("text/html");
        return ([(CONTENT_TYPE, value)], index.data.to_vec()).into_response();
    }

    (StatusCode::NOT_FOUND, Body::empty()).into_response()
}

pub async fn serve_until_signal<F>(
    addr: SocketAddr,
    state: AppState,
    shutdown: F,
) -> Result<(), std::io::Error>
where
    F: std::future::Future<Output = ()> + Send + 'static,
{
    let listener = tokio::net::TcpListener::bind(addr).await?;
    let local_addr = listener.local_addr()?;
    info!(%local_addr, "listening");
    axum::serve(listener, app(state))
        .with_graceful_shutdown(shutdown)
        .await
}
