//! API error handling.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde_json::json;

pub struct ApiError {
    pub status: StatusCode,
    pub message: String,
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<codeilus_core::CodeilusError> for ApiError {
    fn from(err: codeilus_core::CodeilusError) -> Self {
        use codeilus_core::CodeilusError;
        let status = match &err {
            CodeilusError::NotFound(_) => StatusCode::NOT_FOUND,
            CodeilusError::Validation(_) => StatusCode::BAD_REQUEST,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };
        Self { status, message: err.to_string() }
    }
}
