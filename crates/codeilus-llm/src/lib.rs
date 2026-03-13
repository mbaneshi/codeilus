//! Claude Code CLI interface: spawn subprocess, stream-parse JSON output.

pub mod cli;
pub mod context;
pub mod stream_parser;
pub mod types;

pub use cli::ClaudeCli;
pub use context::build_context;
pub use types::*;

use codeilus_core::CodeilusResult;

/// Check if Claude Code CLI is available.
pub async fn is_available() -> bool {
    ClaudeCli::new().is_available().await
}

/// Send a prompt and get the full response.
pub async fn prompt(request: LlmRequest) -> CodeilusResult<LlmResponse> {
    ClaudeCli::new().prompt(&request).await
}
